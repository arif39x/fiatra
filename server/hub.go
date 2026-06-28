package main

import (
	"bytes"
	"encoding/json"
	"io"
	"log"
	"net/http"
	"strings"
	"sync"
	"time"

	"github.com/gorilla/websocket"
)

var upgrader = websocket.Upgrader{
	CheckOrigin: func(r *http.Request) bool {
		origin := r.Header.Get("Origin")
		if origin == "" {
			return true
		}
		return origin == "http://localhost:5173" || origin == "http://127.0.0.1:5173" || origin == "http://localhost:8080" || origin == "http://127.0.0.1:8080" || strings.HasPrefix(origin, "http://localhost") || strings.HasPrefix(origin, "http://127.0.0.1")
	},
}

var (
	compileMu       sync.Mutex
	compileTimer    *time.Timer
	compileEquation string
)

type Client struct {
	hub  *Hub
	conn *websocket.Conn
	send chan []byte
}

type Hub struct {
	clients    map[*Client]bool
	broadcast  chan []byte
	register   chan *Client
	unregister chan *Client
	mu         sync.Mutex
}

func NewHub() *Hub {
	return &Hub{
		clients:    make(map[*Client]bool),
		broadcast:  make(chan []byte),
		register:   make(chan *Client),
		unregister: make(chan *Client),
	}
}

func (h *Hub) Run() {
	for {
		select {
		case client := <-h.register:
			h.mu.Lock()
			h.clients[client] = true
			h.mu.Unlock()
		case client := <-h.unregister:
			h.mu.Lock()
			if _, ok := h.clients[client]; ok {
				delete(h.clients, client)
				close(client.send)
			}
			h.mu.Unlock()
		case message := <-h.broadcast:
			h.mu.Lock()
			for client := range h.clients {
				select {
				case client.send <- message:
				default:
					close(client.send)
					delete(h.clients, client)
				}
			}
			h.mu.Unlock()
		}
	}
}

func (c *Client) ReadPump() {
	defer func() {
		c.hub.unregister <- c
		c.conn.Close()
	}()
	for {
		_, message, err := c.conn.ReadMessage()
		if err != nil {
			break
		}
		handleClientMessage(c, message)
	}
}

func (c *Client) WritePump() {
	for message := range c.send {
		err := c.conn.WriteMessage(websocket.TextMessage, message)
		if err != nil {
			break
		}
	}
	c.conn.Close()
}

type IncomingMsg struct {
	Equation string `json:"equation"`
}

type CompileRequest struct {
	Equation string `json:"equation"`
}

func handleClientMessage(client *Client, message []byte) {
	var in IncomingMsg
	if err := json.Unmarshal(message, &in); err == nil && in.Equation != "" {
		compileMu.Lock()
		compileEquation = in.Equation
		if compileTimer != nil {
			compileTimer.Stop()
		}
		compileTimer = time.AfterFunc(300*time.Millisecond, func() {
			compileMu.Lock()
			eq := compileEquation
			compileMu.Unlock()
			doCompile(client.hub, eq)
		})
		compileMu.Unlock()
	}
}

func doCompile(hub *Hub, equation string) {
	log.Printf("Compiling equation: %s", equation)
	reqBody, _ := json.Marshal(CompileRequest{Equation: equation})
	resp, err := http.Post("http://127.0.0.1:8081/compile_sdf", "application/json", bytes.NewBuffer(reqBody))
	if err != nil {
		log.Println("Error calling python:", err)
		return
	}
	defer resp.Body.Close()

	body, _ := io.ReadAll(resp.Body)
	var compResp map[string]interface{}
	if err := json.Unmarshal(body, &compResp); err == nil {
		if wgsl, ok := compResp["wgsl"].(string); ok {
			outMsg := map[string]string{
				"wgsl": wgsl,
			}
			outBytes, _ := json.Marshal(outMsg)
			hub.broadcast <- outBytes
		} else if detail, ok := compResp["detail"].(string); ok {
			log.Printf("Compilation failed: %s", detail)
		}
	}
}
