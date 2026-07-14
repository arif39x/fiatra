use std::collections::HashMap;
use egui::{Frame, Margin, Rounding, RichText};

use crate::ui::style::*;

#[derive(Debug, Clone, PartialEq)]
pub enum JobStatus {
    Queued,
    Running,
    Completed,
    Failed(String),
}

#[derive(Debug, Clone)]
pub struct GenerationJob {
    pub id: String,
    pub job_type: String,
    pub status: JobStatus,
    pub progress: f32,
    pub created_at: String,
}

pub struct GenerationStatusPanel {
    pub jobs: HashMap<String, GenerationJob>,
    pub show_panel: bool,
    pub on_cancel: Option<Box<dyn FnMut(String) + Send>>,
}

impl GenerationStatusPanel {
    pub fn new() -> Self {
        Self { jobs: HashMap::new(), show_panel: true, on_cancel: None }
    }

    pub fn add_job(&mut self, id: String, job_type: String) {
        self.jobs.insert(id.clone(), GenerationJob {
            id, job_type, status: JobStatus::Queued, progress: 0.0,
            created_at: chrono::Utc::now().to_rfc3339(),
        });
    }

    pub fn update_progress(&mut self, id: &str, progress: f32) {
        if let Some(job) = self.jobs.get_mut(id) {
            job.status = JobStatus::Running;
            job.progress = progress;
        }
    }

    pub fn complete(&mut self, id: &str) {
        if let Some(job) = self.jobs.get_mut(id) {
            job.status = JobStatus::Completed;
            job.progress = 1.0;
        }
    }

    pub fn fail(&mut self, id: &str, error: String) {
        if let Some(job) = self.jobs.get_mut(id) {
            job.status = JobStatus::Failed(error);
        }
    }

    pub fn draw(&mut self, ui: &mut egui::Ui) {
        ui.label(RichText::new("Generation Jobs").size(12.0).color(TEXT));
        ui.add_space(4.0);

        if self.jobs.is_empty() {
            ui.label(RichText::new("No active jobs").size(11.0).color(TEXT_MUTED));
            return;
        }

        let to_remove: Vec<String> = self.jobs.iter()
            .filter(|(_, j)| matches!(j.status, JobStatus::Completed))
            .map(|(id, _)| id.clone()).collect();
        for id in &to_remove { self.jobs.remove(id); }

        for job in self.jobs.values() {
            Frame::none()
                .fill(BG_CARD)
                .rounding(Rounding::same(6.0))
                .inner_margin(Margin::symmetric(10.0, 6.0))
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.label(RichText::new(&job.job_type).size(11.0).color(TEXT));
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if matches!(job.status, JobStatus::Queued | JobStatus::Running) {
                                if ui.button(RichText::new("✕").size(10.0).color(TEXT_MUTED)).clicked() {
                                    if let Some(ref mut cb) = self.on_cancel { cb(job.id.clone()); }
                                }
                            }
                        });
                    });

                    match &job.status {
                        JobStatus::Queued => {
                            ui.label(RichText::new("Queued...").size(10.0).color(TEXT_MUTED));
                        }
                        JobStatus::Running => {
                            ui.add(egui::ProgressBar::new(job.progress)
                                .animate(true)
                                .desired_width(f32::INFINITY)
                                .fill(ACCENT));
                            ui.label(RichText::new(format!("{:.0}%", job.progress * 100.0)).size(10.0).color(TEXT_MUTED));
                        }
                        JobStatus::Completed => {
                            ui.label(RichText::new("Completed").size(10.0).color(GREEN));
                        }
                        JobStatus::Failed(err) => {
                            ui.label(RichText::new(&format!("Failed: {}", err)).size(10.0).color(RED));
                        }
                    }
                });
            ui.add_space(4.0);
        }
    }
}
