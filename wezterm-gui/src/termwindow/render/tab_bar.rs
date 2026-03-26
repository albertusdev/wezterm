use crate::quad::TripleLayerQuadAllocator;
use crate::termwindow::render::RenderScreenLineParams;
use crate::utilsprites::RenderMetrics;
use config::{ConfigHandle, TabBarPosition};
use mux::renderable::RenderableDimensions;
use wezterm_term::color::ColorAttribute;
use window::color::LinearRgba;

impl crate::TermWindow {
    pub fn paint_tab_bar(&mut self, layers: &mut TripleLayerQuadAllocator) -> anyhow::Result<()> {
        if self.config.effective_use_fancy_tab_bar() {
            if self.fancy_tab_bar.is_none() {
                let palette = self.palette().clone();
                let tab_bar = self.build_fancy_tab_bar(&palette)?;
                self.fancy_tab_bar.replace(tab_bar);
            }

            self.ui_items.append(&mut self.paint_fancy_tab_bar()?);
            return Ok(());
        }

        let border = self.get_os_border();

        let palette = self.palette().clone();
        let tab_bar_height = self.tab_bar_pixel_height()?;
        let tab_bar_y = if self.config.resolved_tab_bar_position().is_bottom() {
            ((self.dimensions.pixel_height as f32) - (tab_bar_height + border.bottom.get() as f32))
                .max(0.)
        } else {
            border.top.get() as f32
        };

        // Register the tab bar location
        self.ui_items.append(&mut self.tab_bar.compute_ui_items(
            tab_bar_y as usize,
            self.render_metrics.cell_size.height as usize,
            self.render_metrics.cell_size.width as usize,
        ));

        let window_is_transparent =
            !self.window_background.is_empty() || self.config.window_background_opacity != 1.0;
        let gl_state = self.render_state.as_ref().unwrap();
        let white_space = gl_state.util_sprites.white_space.texture_coords();
        let filled_box = gl_state.util_sprites.filled_box.texture_coords();
        let default_bg = palette
            .resolve_bg(ColorAttribute::Default)
            .to_linear()
            .mul_alpha(if window_is_transparent {
                0.
            } else {
                self.config.text_background_opacity
            });

        self.render_screen_line(
            RenderScreenLineParams {
                top_pixel_y: tab_bar_y,
                left_pixel_x: 0.,
                pixel_width: self.dimensions.pixel_width as f32,
                stable_line_idx: None,
                line: self.tab_bar.line(),
                selection: 0..0,
                cursor: &Default::default(),
                palette: &palette,
                dims: &RenderableDimensions {
                    cols: self.dimensions.pixel_width
                        / self.render_metrics.cell_size.width as usize,
                    physical_top: 0,
                    scrollback_rows: 0,
                    scrollback_top: 0,
                    viewport_rows: 1,
                    dpi: self.terminal_size.dpi,
                    pixel_height: self.render_metrics.cell_size.height as usize,
                    pixel_width: self.terminal_size.pixel_width,
                    reverse_video: false,
                },
                config: &self.config,
                cursor_border_color: LinearRgba::default(),
                foreground: palette.foreground.to_linear(),
                pane: None,
                is_active: true,
                selection_fg: LinearRgba::default(),
                selection_bg: LinearRgba::default(),
                cursor_fg: LinearRgba::default(),
                cursor_bg: LinearRgba::default(),
                cursor_is_default_color: true,
                white_space,
                filled_box,
                window_is_transparent,
                default_bg,
                style: None,
                font: None,
                use_pixel_positioning: self.config.experimental_pixel_positioning,
                render_metrics: self.render_metrics,
                shape_key: None,
                password_input: false,
            },
            layers,
        )?;

        Ok(())
    }

    pub fn tab_bar_pixel_height_impl(
        config: &ConfigHandle,
        fontconfig: &wezterm_font::FontConfiguration,
        render_metrics: &RenderMetrics,
    ) -> anyhow::Result<f32> {
        if config.effective_use_fancy_tab_bar() {
            let font = fontconfig.title_font()?;
            Ok((font.metrics().cell_height.get() as f32 * 1.75).ceil())
        } else {
            Ok(render_metrics.cell_size.height as f32)
        }
    }

    pub fn tab_bar_pixel_height(&self) -> anyhow::Result<f32> {
        Self::tab_bar_pixel_height_impl(&self.config, &self.fonts, &self.render_metrics)
    }

    pub fn tab_bar_pixel_width_impl(
        config: &ConfigHandle,
        fontconfig: &wezterm_font::FontConfiguration,
        dpi: f32,
        window_pixel_width: f32,
    ) -> anyhow::Result<f32> {
        if !config.resolved_tab_bar_position().is_vertical() {
            return Ok(0.);
        }

        let font = fontconfig.title_font()?;
        let metrics = RenderMetrics::with_font_metrics(&font.metrics());
        if let Some(width) = config.tab_bar_width {
            return Ok(width.evaluate_as_pixels(config::DimensionContext {
                dpi,
                pixel_max: window_pixel_width,
                pixel_cell: metrics.cell_size.width as f32,
            }));
        }
        let extra_cells = if config.show_close_tab_button_in_tabs {
            4.5
        } else {
            3.5
        };
        Ok(((config.tab_max_width as f32 + extra_cells) * metrics.cell_size.width as f32).ceil())
    }

    pub fn tab_bar_pixel_width(&self) -> anyhow::Result<f32> {
        Self::tab_bar_pixel_width_impl(
            &self.config,
            &self.fonts,
            self.dimensions.dpi as f32,
            self.dimensions.pixel_width as f32,
        )
    }

    pub fn tab_bar_pixel_offsets(&self) -> anyhow::Result<(f32, f32, f32, f32)> {
        if !self.show_tab_bar {
            return Ok((0., 0., 0., 0.));
        }

        Ok(match self.config.resolved_tab_bar_position() {
            TabBarPosition::Top => (0., self.tab_bar_pixel_height()?, 0., 0.),
            TabBarPosition::Bottom => (0., 0., 0., self.tab_bar_pixel_height()?),
            TabBarPosition::Left => (self.tab_bar_pixel_width()?, 0., 0., 0.),
            TabBarPosition::Right => (0., 0., self.tab_bar_pixel_width()?, 0.),
        })
    }
}
