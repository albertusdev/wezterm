use crate::customglyph::*;
use crate::tabbar::{TabBarItem, TabEntry};
use crate::termwindow::box_model::*;
use crate::termwindow::render::corners::*;

use crate::termwindow::render::window_buttons::window_button_element;
use crate::termwindow::{TabInformation, UIItem, UIItemType};
use crate::utilsprites::RenderMetrics;
use config::{Dimension, DimensionContext, RgbaColor, TabBarColors, TabBarPosition};
use std::convert::TryFrom;
use std::rc::Rc;
use wezterm_font::LoadedFont;
use wezterm_term::color::{ColorAttribute, ColorPalette};
use window::color::LinearRgba;
use window::{IntegratedTitleButtonAlignment, IntegratedTitleButtonStyle};

const X_BUTTON: &[Poly] = &[
    Poly {
        path: &[
            PolyCommand::MoveTo(BlockCoord::One, BlockCoord::Zero),
            PolyCommand::LineTo(BlockCoord::Zero, BlockCoord::One),
        ],
        intensity: BlockAlpha::Full,
        style: PolyStyle::Outline,
    },
    Poly {
        path: &[
            PolyCommand::MoveTo(BlockCoord::Zero, BlockCoord::Zero),
            PolyCommand::LineTo(BlockCoord::One, BlockCoord::One),
        ],
        intensity: BlockAlpha::Full,
        style: PolyStyle::Outline,
    },
];

const PLUS_BUTTON: &[Poly] = &[
    Poly {
        path: &[
            PolyCommand::MoveTo(BlockCoord::Frac(1, 2), BlockCoord::Zero),
            PolyCommand::LineTo(BlockCoord::Frac(1, 2), BlockCoord::One),
        ],
        intensity: BlockAlpha::Full,
        style: PolyStyle::Outline,
    },
    Poly {
        path: &[
            PolyCommand::MoveTo(BlockCoord::Zero, BlockCoord::Frac(1, 2)),
            PolyCommand::LineTo(BlockCoord::One, BlockCoord::Frac(1, 2)),
        ],
        intensity: BlockAlpha::Full,
        style: PolyStyle::Outline,
    },
];

fn tab_accent_color(tab: Option<&TabInformation>) -> Option<RgbaColor> {
    tab.and_then(|tab| {
        tab.accent_color
            .as_ref()
            .and_then(|color| RgbaColor::try_from(color.clone()).ok())
    })
}

fn tab_activity_color(tab: Option<&TabInformation>) -> Option<RgbaColor> {
    tab.and_then(|tab| {
        tab.activity_color
            .as_ref()
            .and_then(|color| RgbaColor::try_from(color.clone()).ok())
    })
}

fn tab_summary_color(tab: Option<&TabInformation>) -> Option<RgbaColor> {
    tab.and_then(|tab| {
        tab.summary_color
            .as_ref()
            .and_then(|color| RgbaColor::try_from(color.clone()).ok())
    })
}

fn tab_secondary_summary_color(tab: Option<&TabInformation>) -> Option<RgbaColor> {
    tab.and_then(|tab| {
        tab.metadata
            .get("wezterm.summary_secondary_color")
            .and_then(|color| RgbaColor::try_from(color.clone()).ok())
    })
}

fn blend_color(base: RgbaColor, tint: RgbaColor, amount: f32) -> RgbaColor {
    let (br, bg, bb, _) = base.as_rgba_u8();
    let (tr, tg, tb, _) = tint.as_rgba_u8();
    let mix = |base: u8, tint: u8| -> u8 {
        let blended = (base as f32 * (1.0 - amount)) + (tint as f32 * amount);
        blended.round().clamp(0.0, 255.0) as u8
    };
    RgbaColor::from((mix(br, tr), mix(bg, tg), mix(bb, tb)))
}

fn tab_activity_marker(
    font: &Rc<LoadedFont>,
    activity: &str,
    color: Option<&RgbaColor>,
) -> Element {
    let marker_color = color
        .cloned()
        .unwrap_or_else(|| RgbaColor::from((124, 133, 156)));
    Element::new(font, ElementContent::Text(activity.to_string()))
        .line_height(Some(0.95))
        .colors(ElementColors {
            border: BorderColor::default(),
            bg: LinearRgba::TRANSPARENT.into(),
            text: marker_color.to_linear().into(),
        })
        .margin(BoxDimension {
            left: Dimension::Cells(0.0),
            right: Dimension::Cells(0.35),
            top: Dimension::Cells(0.0),
            bottom: Dimension::Cells(0.0),
        })
}

impl crate::TermWindow {
    pub fn invalidate_fancy_tab_bar(&mut self) {
        self.fancy_tab_bar.take();
    }

    pub fn build_fancy_tab_bar(&self, palette: &ColorPalette) -> anyhow::Result<ComputedElement> {
        let position = self.config.resolved_tab_bar_position();
        if position.is_vertical() {
            return self.build_fancy_vertical_tab_bar(palette, position);
        }

        let tab_bar_height = self.tab_bar_pixel_height()?;
        let font = self.fonts.title_font()?;
        let metrics = RenderMetrics::with_font_metrics(&font.metrics());
        let items = self.tab_bar.items();
        let tab_info = self.get_tab_information();
        let colors = self
            .config
            .colors
            .as_ref()
            .and_then(|c| c.tab_bar.as_ref())
            .cloned()
            .unwrap_or_else(TabBarColors::default);

        let mut left_status = vec![];
        let mut left_eles = vec![];
        let mut right_eles = vec![];
        let bar_colors = ElementColors {
            border: BorderColor::default(),
            bg: if self.focused.is_some() {
                self.config.window_frame.active_titlebar_bg
            } else {
                self.config.window_frame.inactive_titlebar_bg
            }
            .to_linear()
            .into(),
            text: if self.focused.is_some() {
                self.config.window_frame.active_titlebar_fg
            } else {
                self.config.window_frame.inactive_titlebar_fg
            }
            .to_linear()
            .into(),
        };

        let item_to_elem = |item: &TabEntry| -> Element {
            let element = Element::with_line(&font, &item.title, palette);
            let tab_accent = match item.item {
                TabBarItem::Tab { tab_idx, .. } => tab_accent_color(tab_info.get(tab_idx)),
                _ => None,
            };

            let bg_color = item
                .title
                .get_cell(0)
                .and_then(|c| match c.attrs().background() {
                    ColorAttribute::Default => None,
                    col => Some(palette.resolve_bg(col)),
                });
            let fg_color = item
                .title
                .get_cell(0)
                .and_then(|c| match c.attrs().foreground() {
                    ColorAttribute::Default => None,
                    col => Some(palette.resolve_fg(col)),
                });

            let new_tab = colors.new_tab();
            let new_tab_hover = colors.new_tab_hover();
            let active_tab = colors.active_tab();

            match item.item {
                TabBarItem::RightStatus | TabBarItem::LeftStatus | TabBarItem::None => element
                    .item_type(UIItemType::TabBar(TabBarItem::None))
                    .line_height(Some(1.75))
                    .margin(BoxDimension {
                        left: Dimension::Cells(0.),
                        right: Dimension::Cells(0.),
                        top: Dimension::Cells(0.0),
                        bottom: Dimension::Cells(0.),
                    })
                    .padding(BoxDimension {
                        left: Dimension::Cells(0.5),
                        right: Dimension::Cells(0.),
                        top: Dimension::Cells(0.),
                        bottom: Dimension::Cells(0.),
                    })
                    .border(BoxDimension::new(Dimension::Pixels(0.)))
                    .colors(bar_colors.clone()),
                TabBarItem::NewTabButton => Element::new(
                    &font,
                    ElementContent::Poly {
                        line_width: metrics.underline_height.max(2),
                        poly: SizedPoly {
                            poly: PLUS_BUTTON,
                            width: Dimension::Pixels(metrics.cell_size.height as f32 / 2.),
                            height: Dimension::Pixels(metrics.cell_size.height as f32 / 2.),
                        },
                    },
                )
                .vertical_align(VerticalAlign::Middle)
                .item_type(UIItemType::TabBar(item.item.clone()))
                .margin(BoxDimension {
                    left: Dimension::Cells(0.5),
                    right: Dimension::Cells(0.),
                    top: Dimension::Cells(0.2),
                    bottom: Dimension::Cells(0.),
                })
                .padding(BoxDimension {
                    left: Dimension::Cells(0.5),
                    right: Dimension::Cells(0.5),
                    top: Dimension::Cells(0.2),
                    bottom: Dimension::Cells(0.25),
                })
                .border(BoxDimension::new(Dimension::Pixels(1.)))
                .colors(ElementColors {
                    border: BorderColor::default(),
                    bg: new_tab.bg_color.to_linear().into(),
                    text: new_tab.fg_color.to_linear().into(),
                })
                .hover_colors(Some(ElementColors {
                    border: BorderColor::default(),
                    bg: new_tab_hover.bg_color.to_linear().into(),
                    text: new_tab_hover.fg_color.to_linear().into(),
                })),
                TabBarItem::Tab { active, .. } if active => {
                    let bg = bg_color.unwrap_or_else(|| active_tab.bg_color.into());
                    let border = tab_accent
                        .clone()
                        .map(|color| color.to_linear())
                        .unwrap_or_else(|| bg.to_linear());
                    element
                        .vertical_align(VerticalAlign::Bottom)
                        .item_type(UIItemType::TabBar(item.item.clone()))
                        .margin(BoxDimension {
                            left: Dimension::Cells(0.),
                            right: Dimension::Cells(0.),
                            top: Dimension::Cells(0.2),
                            bottom: Dimension::Cells(0.),
                        })
                        .padding(BoxDimension {
                            left: Dimension::Cells(0.5),
                            right: Dimension::Cells(0.5),
                            top: Dimension::Cells(0.2),
                            bottom: Dimension::Cells(0.25),
                        })
                        .border(BoxDimension::new(Dimension::Pixels(1.)))
                        .border_corners(Some(Corners {
                            top_left: SizedPoly {
                                width: Dimension::Cells(0.5),
                                height: Dimension::Cells(0.5),
                                poly: TOP_LEFT_ROUNDED_CORNER,
                            },
                            top_right: SizedPoly {
                                width: Dimension::Cells(0.5),
                                height: Dimension::Cells(0.5),
                                poly: TOP_RIGHT_ROUNDED_CORNER,
                            },
                            bottom_left: SizedPoly::none(),
                            bottom_right: SizedPoly::none(),
                        }))
                        .colors(ElementColors {
                            border: BorderColor::new(border),
                            bg: bg.to_linear().into(),
                            text: fg_color
                                .unwrap_or_else(|| active_tab.fg_color.into())
                                .to_linear()
                                .into(),
                        })
                }
                TabBarItem::Tab { .. } => element
                    .vertical_align(VerticalAlign::Bottom)
                    .item_type(UIItemType::TabBar(item.item.clone()))
                    .margin(BoxDimension {
                        left: Dimension::Cells(0.),
                        right: Dimension::Cells(0.),
                        top: Dimension::Cells(0.2),
                        bottom: Dimension::Cells(0.),
                    })
                    .padding(BoxDimension {
                        left: Dimension::Cells(0.5),
                        right: Dimension::Cells(0.5),
                        top: Dimension::Cells(0.2),
                        bottom: Dimension::Cells(0.25),
                    })
                    .border(BoxDimension::new(Dimension::Pixels(1.)))
                    .border_corners(Some(Corners {
                        top_left: SizedPoly {
                            width: Dimension::Cells(0.5),
                            height: Dimension::Cells(0.5),
                            poly: TOP_LEFT_ROUNDED_CORNER,
                        },
                        top_right: SizedPoly {
                            width: Dimension::Cells(0.5),
                            height: Dimension::Cells(0.5),
                            poly: TOP_RIGHT_ROUNDED_CORNER,
                        },
                        bottom_left: SizedPoly {
                            width: Dimension::Cells(0.),
                            height: Dimension::Cells(0.33),
                            poly: &[],
                        },
                        bottom_right: SizedPoly {
                            width: Dimension::Cells(0.),
                            height: Dimension::Cells(0.33),
                            poly: &[],
                        },
                    }))
                    .colors({
                        let inactive_tab = colors.inactive_tab();
                        let bg = bg_color
                            .unwrap_or_else(|| inactive_tab.bg_color.into())
                            .to_linear();
                        let edge = tab_accent
                            .clone()
                            .unwrap_or_else(|| colors.inactive_tab_edge())
                            .to_linear();
                        ElementColors {
                            border: BorderColor {
                                left: bg,
                                right: edge,
                                top: bg,
                                bottom: bg,
                            },
                            bg: bg.into(),
                            text: fg_color
                                .unwrap_or_else(|| inactive_tab.fg_color.into())
                                .to_linear()
                                .into(),
                        }
                    })
                    .hover_colors({
                        let inactive_tab_hover = colors.inactive_tab_hover();
                        let hover_border = tab_accent
                            .clone()
                            .map(|color| color.to_linear())
                            .unwrap_or_else(|| {
                                bg_color
                                    .unwrap_or_else(|| inactive_tab_hover.bg_color.into())
                                    .to_linear()
                            });
                        Some(ElementColors {
                            border: BorderColor::new(hover_border),
                            bg: bg_color
                                .unwrap_or_else(|| inactive_tab_hover.bg_color.into())
                                .to_linear()
                                .into(),
                            text: fg_color
                                .unwrap_or_else(|| inactive_tab_hover.fg_color.into())
                                .to_linear()
                                .into(),
                        })
                    }),
                TabBarItem::WindowButton(button) => window_button_element(
                    button,
                    self.window_state.contains(window::WindowState::MAXIMIZED),
                    &font,
                    &metrics,
                    &self.config,
                ),
            }
        };

        let num_tabs: f32 = items
            .iter()
            .map(|item| match item.item {
                TabBarItem::NewTabButton | TabBarItem::Tab { .. } => 1.,
                _ => 0.,
            })
            .sum();
        let max_tab_width = ((self.dimensions.pixel_width as f32 / num_tabs)
            - (1.5 * metrics.cell_size.width as f32))
            .max(0.);

        // Reserve space for the native titlebar buttons
        if self
            .config
            .window_decorations
            .contains(::window::WindowDecorations::INTEGRATED_BUTTONS)
            && self.config.integrated_title_button_style == IntegratedTitleButtonStyle::MacOsNative
            && !self.window_state.contains(window::WindowState::FULL_SCREEN)
        {
            left_status.push(
                Element::new(&font, ElementContent::Text("".to_string())).margin(BoxDimension {
                    left: Dimension::Cells(4.0), // FIXME: determine exact width of macos ... buttons
                    right: Dimension::Cells(0.),
                    top: Dimension::Cells(0.),
                    bottom: Dimension::Cells(0.),
                }),
            );
        }

        for item in items {
            match item.item {
                TabBarItem::LeftStatus => left_status.push(item_to_elem(item)),
                TabBarItem::None | TabBarItem::RightStatus => right_eles.push(item_to_elem(item)),
                TabBarItem::WindowButton(_) => {
                    if self.config.integrated_title_button_alignment
                        == IntegratedTitleButtonAlignment::Left
                    {
                        left_eles.push(item_to_elem(item))
                    } else {
                        right_eles.push(item_to_elem(item))
                    }
                }
                TabBarItem::Tab { tab_idx, active } => {
                    let mut elem = item_to_elem(item);
                    elem.max_width = Some(Dimension::Pixels(max_tab_width));
                    elem.content = match elem.content {
                        ElementContent::Text(_) => unreachable!(),
                        ElementContent::Poly { .. } => unreachable!(),
                        ElementContent::Children(mut kids) => {
                            if self.config.show_close_tab_button_in_tabs {
                                kids.push(make_x_button(&font, &metrics, &colors, tab_idx, active));
                            }
                            ElementContent::Children(kids)
                        }
                    };
                    left_eles.push(elem);
                }
                _ => left_eles.push(item_to_elem(item)),
            }
        }

        let mut children = vec![];

        if !left_status.is_empty() {
            children.push(
                Element::new(&font, ElementContent::Children(left_status))
                    .colors(bar_colors.clone()),
            );
        }

        let window_buttons_at_left = self
            .config
            .window_decorations
            .contains(window::WindowDecorations::INTEGRATED_BUTTONS)
            && (self.config.integrated_title_button_alignment
                == IntegratedTitleButtonAlignment::Left
                || self.config.integrated_title_button_style
                    == IntegratedTitleButtonStyle::MacOsNative);

        let left_padding = if window_buttons_at_left {
            if self.config.integrated_title_button_style == IntegratedTitleButtonStyle::MacOsNative
            {
                if !self.window_state.contains(window::WindowState::FULL_SCREEN) {
                    Dimension::Pixels(70.0)
                } else {
                    Dimension::Cells(0.5)
                }
            } else {
                Dimension::Pixels(0.0)
            }
        } else {
            Dimension::Cells(0.5)
        };

        children.push(
            Element::new(&font, ElementContent::Children(left_eles))
                .vertical_align(VerticalAlign::Bottom)
                .colors(bar_colors.clone())
                .padding(BoxDimension {
                    left: left_padding,
                    right: Dimension::Cells(0.),
                    top: Dimension::Cells(0.),
                    bottom: Dimension::Cells(0.),
                })
                .zindex(1),
        );
        children.push(
            Element::new(&font, ElementContent::Children(right_eles))
                .colors(bar_colors.clone())
                .float(Float::Right),
        );

        let content = ElementContent::Children(children);

        let tabs = Element::new(&font, content)
            .display(DisplayType::Block)
            .item_type(UIItemType::TabBar(TabBarItem::None))
            .min_width(Some(Dimension::Pixels(self.dimensions.pixel_width as f32)))
            .min_height(Some(Dimension::Pixels(tab_bar_height)))
            .vertical_align(VerticalAlign::Bottom)
            .colors(bar_colors);

        let border = self.get_os_border();

        let mut computed = self.compute_element(
            &LayoutContext {
                height: DimensionContext {
                    dpi: self.dimensions.dpi as f32,
                    pixel_max: self.dimensions.pixel_height as f32,
                    pixel_cell: metrics.cell_size.height as f32,
                },
                width: DimensionContext {
                    dpi: self.dimensions.dpi as f32,
                    pixel_max: self.dimensions.pixel_width as f32,
                    pixel_cell: metrics.cell_size.width as f32,
                },
                bounds: euclid::rect(
                    border.left.get() as f32,
                    0.,
                    self.dimensions.pixel_width as f32 - (border.left + border.right).get() as f32,
                    tab_bar_height,
                ),
                metrics: &metrics,
                gl_state: self.render_state.as_ref().unwrap(),
                zindex: 10,
            },
            &tabs,
        )?;

        computed.translate(euclid::vec2(
            0.,
            if self.config.resolved_tab_bar_position().is_bottom() {
                self.dimensions.pixel_height as f32
                    - (computed.bounds.height() + border.bottom.get() as f32)
            } else {
                border.top.get() as f32
            },
        ));

        Ok(computed)
    }

    fn build_fancy_vertical_tab_bar(
        &self,
        palette: &ColorPalette,
        position: TabBarPosition,
    ) -> anyhow::Result<ComputedElement> {
        let tab_bar_width = self.tab_bar_pixel_width()?;
        let font = self.fonts.title_font()?;
        let metrics = RenderMetrics::with_font_metrics(&font.metrics());
        let items = self.tab_bar.items();
        let tab_info = self.get_tab_information();
        let colors = self
            .config
            .colors
            .as_ref()
            .and_then(|c| c.tab_bar.as_ref())
            .cloned()
            .unwrap_or_else(TabBarColors::default);
        let left_side = position.is_left();
        let available_height = self.dimensions.pixel_height as f32
            - self.get_os_border().top.get() as f32
            - self.get_os_border().bottom.get() as f32;
        let content_width = (tab_bar_width - metrics.cell_size.width as f32).max(0.);

        let bar_colors = ElementColors {
            border: BorderColor::default(),
            bg: if self.focused.is_some() {
                self.config.window_frame.active_titlebar_bg
            } else {
                self.config.window_frame.inactive_titlebar_bg
            }
            .to_linear()
            .into(),
            text: if self.focused.is_some() {
                self.config.window_frame.active_titlebar_fg
            } else {
                self.config.window_frame.inactive_titlebar_fg
            }
            .to_linear()
            .into(),
        };

        let status_item = |item: &TabEntry| {
            Element::with_line(&font, &item.title, palette)
                .display(DisplayType::Block)
                .item_type(UIItemType::TabBar(TabBarItem::None))
                .line_height(Some(1.2))
                .margin(BoxDimension {
                    left: Dimension::Cells(0.2),
                    right: Dimension::Cells(0.2),
                    top: Dimension::Cells(0.0),
                    bottom: Dimension::Cells(0.15),
                })
                .padding(BoxDimension {
                    left: Dimension::Cells(0.45),
                    right: Dimension::Cells(0.45),
                    top: Dimension::Cells(0.15),
                    bottom: Dimension::Cells(0.15),
                })
                .colors(bar_colors.clone())
                .max_width(Some(Dimension::Pixels(content_width)))
                .min_width(Some(Dimension::Pixels(content_width)))
        };

        let new_tab_item = |item: &TabEntry| {
            let new_tab = colors.new_tab();
            let new_tab_hover = colors.new_tab_hover();
            Element::new(
                &font,
                ElementContent::Poly {
                    line_width: metrics.underline_height.max(2),
                    poly: SizedPoly {
                        poly: PLUS_BUTTON,
                        width: Dimension::Pixels(metrics.cell_size.height as f32 / 2.),
                        height: Dimension::Pixels(metrics.cell_size.height as f32 / 2.),
                    },
                },
            )
            .display(DisplayType::Block)
            .vertical_align(VerticalAlign::Middle)
            .item_type(UIItemType::TabBar(item.item.clone()))
            .margin(BoxDimension {
                left: Dimension::Cells(0.2),
                right: Dimension::Cells(0.2),
                top: Dimension::Cells(0.2),
                bottom: Dimension::Cells(0.0),
            })
            .padding(BoxDimension {
                left: Dimension::Cells(0.5),
                right: Dimension::Cells(0.5),
                top: Dimension::Cells(0.35),
                bottom: Dimension::Cells(0.35),
            })
            .border(BoxDimension::new(Dimension::Pixels(1.)))
            .colors(ElementColors {
                border: BorderColor::default(),
                bg: new_tab.bg_color.to_linear().into(),
                text: new_tab.fg_color.to_linear().into(),
            })
            .hover_colors(Some(ElementColors {
                border: BorderColor::default(),
                bg: new_tab_hover.bg_color.to_linear().into(),
                text: new_tab_hover.fg_color.to_linear().into(),
            }))
            .min_width(Some(Dimension::Pixels(content_width)))
        };

        let tab_chip = |text: &str,
                        color: Option<&RgbaColor>,
                        default_bg: RgbaColor,
                        default_fg: RgbaColor| {
            let bg = color.cloned().unwrap_or(default_bg);
            Element::new(&font, ElementContent::Text(text.to_string()))
                .margin(BoxDimension {
                    left: Dimension::Cells(0.0),
                    right: Dimension::Cells(0.35),
                    top: Dimension::Cells(0.0),
                    bottom: Dimension::Cells(0.0),
                })
                .padding(BoxDimension {
                    left: Dimension::Cells(0.35),
                    right: Dimension::Cells(0.35),
                    top: Dimension::Cells(0.05),
                    bottom: Dimension::Cells(0.05),
                })
                .border(BoxDimension::new(Dimension::Pixels(1.)))
                .border_corners(Some(Corners {
                    top_left: SizedPoly {
                        width: Dimension::Cells(0.35),
                        height: Dimension::Cells(0.35),
                        poly: TOP_LEFT_ROUNDED_CORNER,
                    },
                    bottom_left: SizedPoly {
                        width: Dimension::Cells(0.35),
                        height: Dimension::Cells(0.35),
                        poly: BOTTOM_LEFT_ROUNDED_CORNER,
                    },
                    top_right: SizedPoly {
                        width: Dimension::Cells(0.35),
                        height: Dimension::Cells(0.35),
                        poly: TOP_RIGHT_ROUNDED_CORNER,
                    },
                    bottom_right: SizedPoly {
                        width: Dimension::Cells(0.35),
                        height: Dimension::Cells(0.35),
                        poly: BOTTOM_RIGHT_ROUNDED_CORNER,
                    },
                }))
                .colors(ElementColors {
                    border: BorderColor::new(bg.to_linear()),
                    bg: bg.to_linear().into(),
                    text: default_fg.to_linear().into(),
                })
        };

        let tab_accent_bar = |color: RgbaColor| {
            Element::new(&font, ElementContent::Text(" ".to_string()))
                .margin(BoxDimension {
                    left: if left_side {
                        Dimension::Cells(0.0)
                    } else {
                        Dimension::Cells(0.35)
                    },
                    right: if left_side {
                        Dimension::Cells(0.45)
                    } else {
                        Dimension::Cells(0.0)
                    },
                    top: Dimension::Cells(0.0),
                    bottom: Dimension::Cells(0.0),
                })
                .padding(BoxDimension {
                    left: Dimension::Cells(0.0),
                    right: Dimension::Cells(0.0),
                    top: Dimension::Cells(0.2),
                    bottom: Dimension::Cells(0.2),
                })
                .min_width(Some(Dimension::Cells(0.22)))
                .border(BoxDimension::new(Dimension::Pixels(0.)))
                .border_corners(Some(Corners {
                    top_left: SizedPoly {
                        width: Dimension::Cells(0.22),
                        height: Dimension::Cells(0.22),
                        poly: TOP_LEFT_ROUNDED_CORNER,
                    },
                    bottom_left: SizedPoly {
                        width: Dimension::Cells(0.22),
                        height: Dimension::Cells(0.22),
                        poly: BOTTOM_LEFT_ROUNDED_CORNER,
                    },
                    top_right: SizedPoly {
                        width: Dimension::Cells(0.22),
                        height: Dimension::Cells(0.22),
                        poly: TOP_RIGHT_ROUNDED_CORNER,
                    },
                    bottom_right: SizedPoly {
                        width: Dimension::Cells(0.22),
                        height: Dimension::Cells(0.22),
                        poly: BOTTOM_RIGHT_ROUNDED_CORNER,
                    },
                }))
                .colors(ElementColors {
                    border: BorderColor::new(color.to_linear()),
                    bg: color.to_linear().into(),
                    text: color.to_linear().into(),
                })
        };

        let tab_item = |item: &TabEntry, tab: &TabInformation, active: bool| {
            let accent = tab_accent_color(Some(tab));
            let activity_color = tab_activity_color(Some(tab));
            let summary_color = tab_summary_color(Some(tab));
            let activity = tab
                .activity
                .as_deref()
                .or_else(|| tab.metadata.get("agent_hud.activity").map(String::as_str))
                .map(str::trim)
                .filter(|value| !value.is_empty());
            let summary = tab
                .summary
                .as_deref()
                .map(str::trim)
                .filter(|value| !value.is_empty());
            let summary_secondary = tab
                .metadata
                .get("wezterm.summary_secondary")
                .map(String::as_str)
                .map(str::trim)
                .filter(|value| !value.is_empty());
            let subtitle = tab
                .subtitle
                .as_deref()
                .or_else(|| tab.metadata.get("agent_hud.subtitle").map(String::as_str));
            let title_max_width = (content_width - (metrics.cell_size.width as f32 * 3.5))
                .max(metrics.cell_size.width as f32 * 4.0);

            let mut rows = vec![];

            let mut title_row_kids = vec![];
            if let Some(color) = accent.clone() {
                let bar = tab_accent_bar(color);
                if left_side {
                    title_row_kids.push(bar);
                }
            }
            if let Some(activity) = activity {
                title_row_kids.push(tab_activity_marker(
                    &font,
                    activity,
                    activity_color.as_ref(),
                ));
            }
            title_row_kids.push(
                Element::with_line(&font, &item.title, palette)
                    .line_height(Some(1.05))
                    .max_width(Some(Dimension::Pixels(title_max_width))),
            );
            if self.config.show_close_tab_button_in_tabs {
                title_row_kids.push(make_x_button(
                    &font,
                    &metrics,
                    &colors,
                    tab.tab_index,
                    active,
                ));
            }
            if let Some(color) = accent.clone() {
                if !left_side {
                    title_row_kids.push(tab_accent_bar(color));
                }
            }
            rows.push(
                Element::new(&font, ElementContent::Children(title_row_kids))
                    .display(DisplayType::Block)
                    .line_height(Some(1.05)),
            );

            let mut meta_kids = vec![];
            if let Some(badge) = tab.badge.as_deref() {
                meta_kids.push(tab_chip(
                    badge,
                    tab.badge_color
                        .as_ref()
                        .and_then(|value| RgbaColor::try_from(value.clone()).ok())
                        .as_ref(),
                    colors.new_tab().bg_color,
                    colors.new_tab().fg_color,
                ));
            }
            if let Some(summary) = summary {
                meta_kids.push(tab_chip(
                    summary,
                    summary_color.as_ref(),
                    colors.new_tab().bg_color,
                    colors.new_tab().fg_color,
                ));
            }
            if let Some(summary_secondary) = summary_secondary {
                meta_kids.push(tab_chip(
                    summary_secondary,
                    tab_secondary_summary_color(Some(tab)).as_ref(),
                    colors.inactive_tab_hover().bg_color,
                    colors.new_tab().fg_color,
                ));
            }
            if let Some(notification) = tab.notification.as_deref() {
                meta_kids.push(
                    tab_chip(
                        notification,
                        tab.notification_color
                            .as_ref()
                            .and_then(|value| RgbaColor::try_from(value.clone()).ok())
                            .as_ref(),
                        RgbaColor::from((209, 77, 65)),
                        colors.active_tab().fg_color,
                    )
                    .float(Float::Right)
                    .margin(BoxDimension {
                        left: Dimension::Cells(0.35),
                        right: Dimension::Cells(0.0),
                        top: Dimension::Cells(0.0),
                        bottom: Dimension::Cells(0.0),
                    }),
                );
            }
            if !meta_kids.is_empty() {
                rows.push(
                    Element::new(&font, ElementContent::Children(meta_kids))
                        .display(DisplayType::Block)
                        .margin(BoxDimension {
                            left: Dimension::Cells(0.0),
                            right: Dimension::Cells(0.0),
                            top: Dimension::Cells(0.15),
                            bottom: Dimension::Cells(0.0),
                        }),
                );
            }

            if let Some(subtitle) = subtitle {
                if !subtitle.is_empty() {
                    rows.push(
                        Element::new(&font, ElementContent::Text(subtitle.to_string()))
                            .display(DisplayType::Block)
                            .line_height(Some(1.0))
                            .margin(BoxDimension {
                                left: Dimension::Cells(0.0),
                                right: Dimension::Cells(0.0),
                                top: Dimension::Cells(0.15),
                                bottom: Dimension::Cells(0.0),
                            })
                            .max_width(Some(Dimension::Pixels(
                                content_width - (metrics.cell_size.width as f32 * 0.5),
                            ))),
                    );
                }
            }

            let mut elem = Element::new(&font, ElementContent::Children(rows))
                .display(DisplayType::Block)
                .item_type(UIItemType::TabBar(item.item.clone()))
                .margin(BoxDimension {
                    left: Dimension::Cells(0.2),
                    right: Dimension::Cells(0.2),
                    top: Dimension::Cells(0.15),
                    bottom: Dimension::Cells(0.0),
                })
                .padding(BoxDimension {
                    left: Dimension::Cells(0.55),
                    right: Dimension::Cells(0.55),
                    top: Dimension::Cells(0.38),
                    bottom: Dimension::Cells(0.38),
                })
                .border(BoxDimension::new(Dimension::Pixels(1.)))
                .border_corners(Some(if left_side {
                    Corners {
                        top_left: SizedPoly::none(),
                        bottom_left: SizedPoly::none(),
                        top_right: SizedPoly {
                            width: Dimension::Cells(0.5),
                            height: Dimension::Cells(0.5),
                            poly: TOP_RIGHT_ROUNDED_CORNER,
                        },
                        bottom_right: SizedPoly {
                            width: Dimension::Cells(0.5),
                            height: Dimension::Cells(0.5),
                            poly: BOTTOM_RIGHT_ROUNDED_CORNER,
                        },
                    }
                } else {
                    Corners {
                        top_left: SizedPoly {
                            width: Dimension::Cells(0.5),
                            height: Dimension::Cells(0.5),
                            poly: TOP_LEFT_ROUNDED_CORNER,
                        },
                        bottom_left: SizedPoly {
                            width: Dimension::Cells(0.5),
                            height: Dimension::Cells(0.5),
                            poly: BOTTOM_LEFT_ROUNDED_CORNER,
                        },
                        top_right: SizedPoly::none(),
                        bottom_right: SizedPoly::none(),
                    }
                }))
                .min_width(Some(Dimension::Pixels(content_width)))
                .max_width(Some(Dimension::Pixels(content_width)))
                .min_height(Some(Dimension::Cells(if subtitle.is_some() {
                    2.9
                } else if tab.badge.is_some()
                    || tab.notification.is_some()
                    || tab.summary.is_some()
                    || tab.metadata.contains_key("wezterm.summary_secondary")
                {
                    2.25
                } else {
                    1.6
                })));

            elem.colors = if active {
                let active_tab = colors.active_tab();
                let bg = accent
                    .as_ref()
                    .map(|color| blend_color(active_tab.bg_color, color.clone(), 0.18))
                    .unwrap_or(active_tab.bg_color);
                let border = accent
                    .as_ref()
                    .map(|color| blend_color(active_tab.bg_color, color.clone(), 0.38))
                    .unwrap_or(active_tab.bg_color);
                ElementColors {
                    border: BorderColor::new(border.to_linear()),
                    bg: bg.to_linear().into(),
                    text: active_tab.fg_color.to_linear().into(),
                }
            } else {
                let inactive_tab = colors.inactive_tab();
                let bg = accent
                    .as_ref()
                    .map(|color| blend_color(inactive_tab.bg_color, color.clone(), 0.14))
                    .unwrap_or(inactive_tab.bg_color);
                let border = accent
                    .as_ref()
                    .map(|color| blend_color(colors.inactive_tab_edge(), color.clone(), 0.46))
                    .unwrap_or(colors.inactive_tab_edge());
                ElementColors {
                    border: BorderColor::new(border.to_linear()),
                    bg: bg.to_linear().into(),
                    text: inactive_tab.fg_color.to_linear().into(),
                }
            };
            elem.hover_colors = if active {
                None
            } else {
                let inactive_tab_hover = colors.inactive_tab_hover();
                let bg = accent
                    .as_ref()
                    .map(|color| blend_color(inactive_tab_hover.bg_color, color.clone(), 0.12))
                    .unwrap_or(inactive_tab_hover.bg_color);
                Some(ElementColors {
                    border: BorderColor::new(bg.to_linear()),
                    bg: bg.to_linear().into(),
                    text: inactive_tab_hover.fg_color.to_linear().into(),
                })
            };

            elem
        };

        let mut header = vec![];
        let mut tabs = vec![];
        let mut footer = vec![];

        for item in items {
            match item.item {
                TabBarItem::LeftStatus | TabBarItem::RightStatus => {
                    if item.title.len() > 0 {
                        header.push(status_item(item));
                    }
                }
                TabBarItem::WindowButton(button) => {
                    header.push(
                        window_button_element(
                            button,
                            self.window_state.contains(window::WindowState::MAXIMIZED),
                            &font,
                            &metrics,
                            &self.config,
                        )
                        .display(DisplayType::Block)
                        .margin(BoxDimension {
                            left: Dimension::Cells(0.2),
                            right: Dimension::Cells(0.2),
                            top: Dimension::Cells(0.15),
                            bottom: Dimension::Cells(0.0),
                        }),
                    );
                }
                TabBarItem::Tab { tab_idx, active } => {
                    tabs.push(tab_item(item, &tab_info[tab_idx], active))
                }
                TabBarItem::NewTabButton => footer.push(new_tab_item(item)),
                TabBarItem::None => {}
            }
        }

        let mut children = vec![];

        if !header.is_empty() {
            children.push(
                Element::new(&font, ElementContent::Children(header))
                    .display(DisplayType::Block)
                    .colors(bar_colors.clone())
                    .padding(BoxDimension {
                        left: Dimension::Cells(0.15),
                        right: Dimension::Cells(0.15),
                        top: Dimension::Cells(0.15),
                        bottom: Dimension::Cells(0.2),
                    }),
            );
        }

        children.push(
            Element::new(&font, ElementContent::Children(tabs))
                .display(DisplayType::Block)
                .colors(bar_colors.clone())
                .padding(BoxDimension {
                    left: Dimension::Cells(0.15),
                    right: Dimension::Cells(0.15),
                    top: Dimension::Cells(0.15),
                    bottom: Dimension::Cells(0.15),
                }),
        );

        if !footer.is_empty() {
            children.push(
                Element::new(&font, ElementContent::Children(footer))
                    .display(DisplayType::Block)
                    .colors(bar_colors.clone())
                    .padding(BoxDimension {
                        left: Dimension::Cells(0.15),
                        right: Dimension::Cells(0.15),
                        top: Dimension::Cells(0.2),
                        bottom: Dimension::Cells(0.15),
                    }),
            );
        }

        let rail = Element::new(&font, ElementContent::Children(children))
            .display(DisplayType::Block)
            .item_type(UIItemType::TabBar(TabBarItem::None))
            .min_width(Some(Dimension::Pixels(tab_bar_width)))
            .min_height(Some(Dimension::Pixels(available_height)))
            .colors(bar_colors);

        let border = self.get_os_border();
        let mut computed = self.compute_element(
            &LayoutContext {
                height: DimensionContext {
                    dpi: self.dimensions.dpi as f32,
                    pixel_max: self.dimensions.pixel_height as f32,
                    pixel_cell: metrics.cell_size.height as f32,
                },
                width: DimensionContext {
                    dpi: self.dimensions.dpi as f32,
                    pixel_max: self.dimensions.pixel_width as f32,
                    pixel_cell: metrics.cell_size.width as f32,
                },
                bounds: euclid::rect(0., 0., tab_bar_width, available_height),
                metrics: &metrics,
                gl_state: self.render_state.as_ref().unwrap(),
                zindex: 10,
            },
            &rail,
        )?;

        computed.translate(euclid::vec2(
            if left_side {
                border.left.get() as f32
            } else {
                self.dimensions.pixel_width as f32
                    - (computed.bounds.width() + border.right.get() as f32)
            },
            border.top.get() as f32,
        ));

        Ok(computed)
    }

    pub fn paint_fancy_tab_bar(&self) -> anyhow::Result<Vec<UIItem>> {
        let computed = self.fancy_tab_bar.as_ref().ok_or_else(|| {
            anyhow::anyhow!("paint_fancy_tab_bar called but fancy_tab_bar is None")
        })?;
        let mut ui_items = computed.ui_items();

        if self.config.resolved_tab_bar_position().is_vertical() {
            let handle_width = 6usize;
            let x = if self.config.resolved_tab_bar_position().is_left() {
                computed.bounds.max_x().round() as isize - (handle_width as isize / 2)
            } else {
                computed.bounds.min_x().round() as isize - (handle_width as isize / 2)
            };
            ui_items.push(UIItem {
                x: x.max(0) as usize,
                y: computed.bounds.min_y().round().max(0.) as usize,
                width: handle_width,
                height: computed.bounds.height().round().max(0.) as usize,
                item_type: UIItemType::TabBarResizeHandle,
            });
        }

        let gl_state = self.render_state.as_ref().unwrap();
        self.render_element(&computed, gl_state, None)?;

        Ok(ui_items)
    }
}

fn make_x_button(
    font: &Rc<LoadedFont>,
    metrics: &RenderMetrics,
    colors: &TabBarColors,
    tab_idx: usize,
    active: bool,
) -> Element {
    Element::new(
        &font,
        ElementContent::Poly {
            line_width: metrics.underline_height.max(2),
            poly: SizedPoly {
                poly: X_BUTTON,
                width: Dimension::Pixels(metrics.cell_size.height as f32 / 2.),
                height: Dimension::Pixels(metrics.cell_size.height as f32 / 2.),
            },
        },
    )
    // Ensure that we draw our background over the
    // top of the rest of the tab contents
    .zindex(1)
    .vertical_align(VerticalAlign::Middle)
    .float(Float::Right)
    .item_type(UIItemType::CloseTab(tab_idx))
    .hover_colors({
        let inactive_tab_hover = colors.inactive_tab_hover();
        let active_tab = colors.active_tab();

        Some(ElementColors {
            border: BorderColor::default(),
            bg: (if active {
                inactive_tab_hover.bg_color
            } else {
                active_tab.bg_color
            })
            .to_linear()
            .into(),
            text: (if active {
                inactive_tab_hover.fg_color
            } else {
                active_tab.fg_color
            })
            .to_linear()
            .into(),
        })
    })
    .padding(BoxDimension {
        left: Dimension::Cells(0.25),
        right: Dimension::Cells(0.25),
        top: Dimension::Cells(0.25),
        bottom: Dimension::Cells(0.25),
    })
    .margin(BoxDimension {
        left: Dimension::Cells(0.5),
        right: Dimension::Cells(0.),
        top: Dimension::Cells(0.),
        bottom: Dimension::Cells(0.),
    })
}
