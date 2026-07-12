use crate::core::ecs::{EntityId, MaterialComponent, TransformComponent};

#[derive(Clone)]
pub struct TransformSnapshot {
    pub entity: EntityId,
    pub prev: TransformComponent,
    pub current: TransformComponent,
}

#[derive(Clone)]
pub struct MaterialSnapshot {
    pub entity: EntityId,
    pub prev: MaterialComponent,
    pub current: MaterialComponent,
}

#[derive(Clone)]
pub enum EditCommand {
    Transform(TransformSnapshot),
    Material(MaterialSnapshot),
}

pub struct UndoStack {
    commands: Vec<EditCommand>,
    current: usize,
    max_size: usize,
}

impl UndoStack {
    pub fn new() -> Self {
        Self {
            commands: Vec::new(),
            current: 0,
            max_size: 100,
        }
    }

    pub fn push(&mut self, cmd: EditCommand) {
        self.commands.truncate(self.current);
        self.commands.push(cmd);
        if self.commands.len() > self.max_size {
            self.commands.remove(0);
        }
        self.current = self.commands.len();
    }

    pub fn undo(&mut self) -> Option<&EditCommand> {
        if self.current > 0 {
            self.current -= 1;
            Some(&self.commands[self.current])
        } else {
            None
        }
    }

    pub fn redo(&mut self) -> Option<&EditCommand> {
        if self.current < self.commands.len() {
            let cmd = &self.commands[self.current];
            self.current += 1;
            Some(cmd)
        } else {
            None
        }
    }

    pub fn can_undo(&self) -> bool {
        self.current > 0
    }

    pub fn can_redo(&self) -> bool {
        self.current < self.commands.len()
    }
}
