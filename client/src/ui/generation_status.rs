use std::collections::HashMap;

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
    #[allow(dead_code)]
    pub created_at: String,
}

pub struct GenerationStatusPanel {
    pub jobs: HashMap<String, GenerationJob>,
    pub show_panel: bool,
    pub on_cancel: Option<Box<dyn FnMut(String) + Send>>,
}

impl GenerationStatusPanel {
    pub fn new() -> Self {
        Self {
            jobs: HashMap::new(),
            show_panel: true,
            on_cancel: None,
        }
    }

    pub fn add_job(&mut self, id: String, job_type: String) {
        self.jobs.insert(
            id.clone(),
            GenerationJob {
                id,
                job_type,
                status: JobStatus::Queued,
                progress: 0.0,
                created_at: chrono::Utc::now().to_rfc3339(),
            },
        );
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

    pub fn draw(&mut self, ctx: &egui::Context) {
        if !self.show_panel {
            return;
        }
        egui::Window::new("Generation Jobs")
            .id(egui::Id::new("generation_jobs"))
            .default_width(300.0)
            .show(ctx, |ui| {
                if self.jobs.is_empty() {
                    ui.label("No active jobs.");
                    return;
                }
                let to_remove: Vec<String> = self
                    .jobs
                    .iter()
                    .filter(|(_, j)| matches!(j.status, JobStatus::Completed))
                    .map(|(id, _)| id.clone())
                    .collect();
                for id in &to_remove {
                    self.jobs.remove(id);
                }
                for job in self.jobs.values() {
                    ui.group(|ui| {
                        ui.horizontal(|ui| {
                            ui.label(format!("[{}] {}", job.job_type, job.id));
                            if matches!(job.status, JobStatus::Queued | JobStatus::Running) {
                                if ui.button("Cancel").clicked() {
                                    if let Some(ref mut cb) = self.on_cancel {
                                        cb(job.id.clone());
                                    }
                                }
                            }
                        });
                        match &job.status {
                            JobStatus::Queued => {
                                ui.label("Queued...");
                            }
                            JobStatus::Running => {
                                ui.label(format!("Progress: {:.0}%", job.progress * 100.0));
                                ui.add(
                                    egui::ProgressBar::new(job.progress)
                                        .animate(true),
                                );
                            }
                            JobStatus::Completed => {
                                ui.label("Completed!");
                            }
                            JobStatus::Failed(err) => {
                                ui.colored_label(
                                    egui::Color32::RED,
                                    format!("Failed: {}", err),
                                );
                            }
                        }
                    });
                }
            });
    }
}
