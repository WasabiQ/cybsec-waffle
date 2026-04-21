use eframe::egui;
use egui::{Color32, RichText, Vec2, Visuals};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::{Command, Child};

// --- Constants & Styling ---
const BG_MAIN: Color32 = Color32::from_rgb(5, 12, 20);
const BG_SIDEBAR: Color32 = Color32::from_rgb(2, 8, 16);
const ACCENT: Color32 = Color32::from_rgb(122, 162, 247); // Tokyo Night Blue
const PORT_RANGE: std::ops::RangeInclusive<u16> = 3201..=3280;

#[derive(PartialEq)]
enum NavTab { Challenges, Tools, Logs }

struct ActiveChallenge {
    process: Child,
    port: u16,
    name: String,
}

struct CybsecWaffle {
    current_tab: NavTab,
    challenges_path: PathBuf,
    active_instances: HashMap<u16, ActiveChallenge>,
    logs: Vec<String>,
}

impl CybsecWaffle {
    fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        // Path Resolution logic
        let exe_path = std::env::current_exe().unwrap_or_default();
        let challenges_path = exe_path.parent().unwrap_or(Path::new(".")).join("challenges");

        Self {
            current_tab: NavTab::Challenges,
            challenges_path,
            active_instances: HashMap::new(),
            logs: vec!["[SYSTEM] CSW Orchestrator Initialized...".to_string()],
        }
    }

    fn log(&mut self, msg: &str) {
        self.logs.push(format!("[{}] {}", chrono::Local::now().format("%H:%M:%S"), msg));
    }

    /// Spawns a Docker container using runsc (gVisor) for isolation
    fn launch_sandbox(&mut self, challenge_name: &str) {
        // Find next available port
        let port = PORT_RANGE.into_iter()
            .find(|p| !self.active_instances.contains_key(p))
            .unwrap_or(3201);

        self.log(&format!("Launching sandbox for {} on port {}", challenge_name, port));

        // Logic: docker run --runtime=runsc -p 32xx:80 cybsec/lab_name
        let child = Command::new("docker")
            .args([
                "run", "--rm", "-d",
                "--runtime", "runsc", // gVisor integration
                "-p", &format!("{}:80", port),
                "--name", &format!("csw_{}", port),
                challenge_name
            ])
            .spawn();

        match child {
            Ok(c) => {
                self.active_instances.insert(port, ActiveChallenge {
                    process: c,
                    port,
                    name: challenge_name.to_string(),
                });
                // Open browser automatically
                let _ = open::that(format!("http://localhost:{}", port));
            }
            Err(e) => self.log(&format!("Error: {}", e)),
        }
    }

    fn open_shell(&mut self) {
        // Spawns an external terminal with toolset (assuming xterm or gnome-terminal)
        let _ = Command::new("gnome-terminal")
            .args(["--", "bash", "-c", "echo 'Cybsec Waffle Shell'; exec bash"])
            .spawn();
        self.log("External shell initialized.");
    }
}

impl eframe::App for CybsecWaffle {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Apply Custom Tokyo Night Theme
        let mut visuals = Visuals::dark();
        visuals.widgets.noninteractive.bg_fill = BG_MAIN;
        visuals.widgets.active.fg_stroke = egui::Stroke::new(2.0, ACCENT);
        ctx.set_visuals(visuals);

        // Sidebar Implementation
        egui::SidePanel::left("sidebar")
            .resizable(false)
            .default_width(80.0)
            .frame(egui::Frame::none().fill(BG_SIDEBAR))
            .show(ctx, |ui| {
                ui.vertical_centered(|ui| {
                    ui.add_space(20.0);
                    
                    // Nav Icons
                    if ui.button(RichText::new("󱇬").size(24.0)).clicked() { self.current_tab = NavTab::Challenges; }
                    ui.add_space(15.0);
                    if ui.button(RichText::new("󰆍").size(24.0)).clicked() { self.current_tab = NavTab::Tools; }
                    ui.add_space(15.0);
                    if ui.button(RichText::new("󰄱").size(24.0)).clicked() { self.current_tab = NavTab::Logs; }

                    // Profile Section (Bottom)
                    ui.with_layout(egui::Layout::bottom_up(egui::Align::Center), |ui| {
                        ui.add_space(20.0);
                        ui.label(RichText::new("Operator").small().color(Color32::DARK_GRAY));
                        ui.painter().circle_filled(ui.cursor().center() - Vec2::new(0.0, 30.0), 18.0, Color32::from_rgb(30, 30, 45));
                        ui.add_space(10.0);
                    });
                });
            });

        // Main Workspace
        egui::CentralPanel::default().show(ctx, |ui| {
            match self.current_tab {
                NavTab::Challenges => {
                    ui.heading("Cybersecurity Lab Coordinator");
                    ui.label(format!("Scanning path: {:?}", self.challenges_path));
                    ui.separator();

                    // Example Challenge Cards
                    ui.group(|ui| {
                        ui.horizontal(|ui| {
                            ui.vertical(|ui| {
                                ui.label(RichText::new("SQLi_Vulnerable_Web_01.csw").strong());
                                ui.label("Type: Web Exploitation | Isolation: gVisor");
                            });
                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                if ui.button("Launch Sandbox").clicked() {
                                    self.launch_sandbox("sqli-lab-image");
                                }
                                if ui.button("Open Shell").clicked() {
                                    self.open_shell();
                                }
                            });
                        });
                    });

                    ui.add_space(20.0);
                    ui.heading("Active Deployments");
                    for (port, instance) in &self.active_instances {
                        ui.colored_label(ACCENT, format!("Port {}: {} [RUNNING]", port, instance.name));
                    }
                }
                NavTab::Logs => {
                    egui::ScrollArea::vertical().stick_to_bottom(true).show(ui, |ui| {
                        for log in &self.logs {
                            ui.label(RichText::new(log).monospace().size(12.0));
                        }
                    });
                }
                _ => { ui.label("Module under development..."); }
            }
        });
    }
}

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([900.0, 600.0])
            .with_title("Cybsec Waffle"),
        ..Default::default()
    };
    
    eframe::run_native(
        "Cybsec Waffle",
        options,
        Box::new(|cc| Box::new(CybsecWaffle::new(cc))),
    )
}
