mod clocks_frame;
mod oc_adjustment;
mod performance_frame;
mod power_cap_frame;
mod stats_frame;

use clocks_frame::ClocksFrame;
use gtk::prelude::*;
use gtk::*;
use lact_client::schema::{
    amdgpu_sysfs::gpu_handle::{overdrive::ClocksTableGen, PerformanceLevel},
    DeviceStats, SystemInfo,
};
use performance_frame::PerformanceFrame;
use power_cap_frame::PowerCapFrame;
use stats_frame::StatsFrame;
use tracing::warn;

const OVERCLOCKING_DISABLED_TEXT: &str = "Overclocking support is not enabled! \
You can still change basic settings, but the more advanced clocks and voltage control will not be available.";

#[derive(Clone)]
pub struct OcPage {
    pub container: ScrolledWindow,
    stats_frame: StatsFrame,
    pub performance_frame: PerformanceFrame,
    power_cap_frame: PowerCapFrame,
    pub clocks_frame: ClocksFrame,
    pub enable_overclocking_button: Option<Button>,
}

impl OcPage {
    pub fn new(system_info: &SystemInfo) -> Self {
        let container = ScrolledWindow::builder()
            .hscrollbar_policy(PolicyType::Never)
            .build();

        let vbox = Box::builder()
            .orientation(Orientation::Vertical)
            .spacing(15)
            .margin_start(5)
            .margin_end(5)
            .build();

        let mut enable_overclocking_button = None;

        if system_info.amdgpu_overdrive_enabled == Some(false) {
            let (warning_frame, button) = oc_warning_frame();
            enable_overclocking_button = Some(button);
            vbox.append(&warning_frame);
        }

        let stats_frame = StatsFrame::new();
        vbox.append(&stats_frame.container);

        let power_cap_frame = PowerCapFrame::new();
        let performance_level_frame = PerformanceFrame::new();
        let clocks_frame = ClocksFrame::new();

        vbox.append(&power_cap_frame.container);
        vbox.append(&performance_level_frame.container);
        vbox.append(&clocks_frame.container);

        container.set_child(Some(&vbox));

        Self {
            container,
            stats_frame,
            performance_frame: performance_level_frame,
            clocks_frame,
            power_cap_frame,
            enable_overclocking_button,
        }
    }

    pub fn set_stats(&self, stats: &DeviceStats, initial: bool) {
        self.stats_frame.set_stats(stats);
        if initial {
            self.power_cap_frame.set_data(
                stats.power.cap_current,
                stats.power.cap_max,
                stats.power.cap_default,
            );
            self.set_performance_level(stats.performance_level);
        }
    }

    pub fn set_clocks_table(&self, table: Option<ClocksTableGen>) {
        match table {
            Some(table) => match self.clocks_frame.set_table(table) {
                Ok(()) => {
                    self.clocks_frame.show();
                }
                Err(err) => {
                    warn!("got invalid clocks table: {err:?}");
                    self.clocks_frame.hide();
                }
            },
            None => {
                self.clocks_frame.hide();
            }
        }
    }

    pub fn connect_settings_changed<F: Fn() + 'static + Clone>(&self, f: F) {
        self.performance_frame.connect_settings_changed(f.clone());
        self.power_cap_frame.connect_cap_changed(f.clone());
        self.clocks_frame.connect_clocks_changed(f);
    }

    pub fn set_performance_level(&self, profile: Option<PerformanceLevel>) {
        match profile {
            Some(profile) => {
                self.performance_frame.show();
                self.performance_frame.set_active_level(profile);
            }
            None => self.performance_frame.hide(),
        }
    }

    pub fn get_performance_level(&self) -> Option<PerformanceLevel> {
        if self.performance_frame.get_visibility() {
            let level = self.performance_frame.get_selected_performance_level();
            Some(level)
        } else {
            None
        }
    }

    pub fn get_power_cap(&self) -> Option<f64> {
        self.power_cap_frame.get_cap()
    }
}

fn oc_warning_frame() -> (Frame, Button) {
    let container = Frame::new(Some("Overclocking information"));

    container.set_label_align(0.3);

    let vbox = Box::builder()
        .orientation(Orientation::Vertical)
        .spacing(5)
        .margin_top(10)
        .margin_bottom(10)
        .margin_start(10)
        .margin_end(10)
        .build();

    let warning_label = Label::builder()
        .use_markup(true)
        .label(OVERCLOCKING_DISABLED_TEXT)
        .wrap(true)
        .wrap_mode(pango::WrapMode::Word)
        .build();

    let enable_button = Button::builder()
        .label("Enable Overclocking")
        .halign(Align::End)
        .build();

    vbox.append(&warning_label);
    vbox.append(&enable_button);

    container.set_child(Some(&vbox));

    (container, enable_button)
}
