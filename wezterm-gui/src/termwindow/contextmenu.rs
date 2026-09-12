use super::box_model::*;
use super::modal::Modal;
use super::{TermWindow, UIItemType};
use crate::customglyph::{BlockAlpha, BlockCoord, Poly, PolyCommand, PolyStyle};
use crate::termwindow::render::corners::{
    BOTTOM_LEFT_ROUNDED_CORNER, BOTTOM_RIGHT_ROUNDED_CORNER, TOP_LEFT_ROUNDED_CORNER,
    TOP_RIGHT_ROUNDED_CORNER,
};
use crate::utilsprites::RenderMetrics;
use config::keyassignment::{
    ClipboardCopyDestination, ClipboardPasteSource, KeyAssignment, SpawnCommand,
};
use config::{DeferredKeyCode, Dimension, DimensionContext, KeyNoAction};
use mux::pane::PaneId;
use mux::Mux;
use std::cell::{Ref, RefCell};
use wezterm_term::{KeyCode, KeyModifiers, MouseEvent};
use window::color::LinearRgba;
use window::{KeyCode as WindowKeyCode, Modifiers as WindowModifiers};

struct MenuItem {
    label: &'static str,
    action: KeyAssignment,
    enabled: bool,
}

const ADD_RIGHT_ICON: &[Poly] = &[
    Poly {
        path: &[
            PolyCommand::MoveTo(BlockCoord::Frac(3, 16), BlockCoord::Frac(1, 16)),
            PolyCommand::LineTo(BlockCoord::Frac(3, 16), BlockCoord::Frac(15, 16)),
            PolyCommand::LineTo(BlockCoord::Frac(11, 16), BlockCoord::Frac(15, 16)),
            PolyCommand::LineTo(BlockCoord::Frac(11, 16), BlockCoord::Frac(9, 16)),
            PolyCommand::LineTo(BlockCoord::Frac(9, 16), BlockCoord::Frac(9, 16)),
            PolyCommand::LineTo(BlockCoord::Frac(9, 16), BlockCoord::Frac(13, 16)),
            PolyCommand::LineTo(BlockCoord::Frac(5, 16), BlockCoord::Frac(13, 16)),
            PolyCommand::LineTo(BlockCoord::Frac(5, 16), BlockCoord::Frac(3, 16)),
            PolyCommand::LineTo(BlockCoord::Frac(9, 16), BlockCoord::Frac(3, 16)),
            PolyCommand::LineTo(BlockCoord::Frac(9, 16), BlockCoord::Frac(1, 16)),
            PolyCommand::Close,
        ],
        intensity: BlockAlpha::Full,
        style: PolyStyle::Fill,
    },
    Poly {
        path: &[
            PolyCommand::MoveTo(BlockCoord::Frac(10, 16), BlockCoord::Frac(2, 16)),
            PolyCommand::LineTo(BlockCoord::Frac(10, 16), BlockCoord::Frac(4, 16)),
            PolyCommand::LineTo(BlockCoord::Frac(8, 16), BlockCoord::Frac(4, 16)),
            PolyCommand::LineTo(BlockCoord::Frac(8, 16), BlockCoord::Frac(6, 16)),
            PolyCommand::LineTo(BlockCoord::Frac(10, 16), BlockCoord::Frac(6, 16)),
            PolyCommand::LineTo(BlockCoord::Frac(10, 16), BlockCoord::Frac(8, 16)),
            PolyCommand::LineTo(BlockCoord::Frac(12, 16), BlockCoord::Frac(8, 16)),
            PolyCommand::LineTo(BlockCoord::Frac(12, 16), BlockCoord::Frac(6, 16)),
            PolyCommand::LineTo(BlockCoord::Frac(14, 16), BlockCoord::Frac(6, 16)),
            PolyCommand::LineTo(BlockCoord::Frac(14, 16), BlockCoord::Frac(4, 16)),
            PolyCommand::LineTo(BlockCoord::Frac(12, 16), BlockCoord::Frac(4, 16)),
            PolyCommand::LineTo(BlockCoord::Frac(12, 16), BlockCoord::Frac(2, 16)),
            PolyCommand::Close,
        ],
        intensity: BlockAlpha::Full,
        style: PolyStyle::Fill,
    },
];

const ADD_DOWN_ICON: &[Poly] = &[
    Poly {
        path: &[
            PolyCommand::MoveTo(BlockCoord::Frac(15, 16), BlockCoord::Frac(3, 16)),
            PolyCommand::LineTo(BlockCoord::Frac(1, 16), BlockCoord::Frac(3, 16)),
            PolyCommand::LineTo(BlockCoord::Frac(1, 16), BlockCoord::Frac(11, 16)),
            PolyCommand::LineTo(BlockCoord::Frac(7, 16), BlockCoord::Frac(11, 16)),
            PolyCommand::LineTo(BlockCoord::Frac(7, 16), BlockCoord::Frac(9, 16)),
            PolyCommand::LineTo(BlockCoord::Frac(3, 16), BlockCoord::Frac(9, 16)),
            PolyCommand::LineTo(BlockCoord::Frac(3, 16), BlockCoord::Frac(5, 16)),
            PolyCommand::LineTo(BlockCoord::Frac(13, 16), BlockCoord::Frac(5, 16)),
            PolyCommand::LineTo(BlockCoord::Frac(13, 16), BlockCoord::Frac(9, 16)),
            PolyCommand::LineTo(BlockCoord::Frac(15, 16), BlockCoord::Frac(9, 16)),
            PolyCommand::Close,
        ],
        intensity: BlockAlpha::Full,
        style: PolyStyle::Fill,
    },
    Poly {
        path: &[
            PolyCommand::MoveTo(BlockCoord::Frac(14, 16), BlockCoord::Frac(10, 16)),
            PolyCommand::LineTo(BlockCoord::Frac(12, 16), BlockCoord::Frac(10, 16)),
            PolyCommand::LineTo(BlockCoord::Frac(12, 16), BlockCoord::Frac(8, 16)),
            PolyCommand::LineTo(BlockCoord::Frac(10, 16), BlockCoord::Frac(8, 16)),
            PolyCommand::LineTo(BlockCoord::Frac(10, 16), BlockCoord::Frac(10, 16)),
            PolyCommand::LineTo(BlockCoord::Frac(8, 16), BlockCoord::Frac(10, 16)),
            PolyCommand::LineTo(BlockCoord::Frac(8, 16), BlockCoord::Frac(12, 16)),
            PolyCommand::LineTo(BlockCoord::Frac(10, 16), BlockCoord::Frac(12, 16)),
            PolyCommand::LineTo(BlockCoord::Frac(10, 16), BlockCoord::Frac(14, 16)),
            PolyCommand::LineTo(BlockCoord::Frac(12, 16), BlockCoord::Frac(14, 16)),
            PolyCommand::LineTo(BlockCoord::Frac(12, 16), BlockCoord::Frac(12, 16)),
            PolyCommand::LineTo(BlockCoord::Frac(14, 16), BlockCoord::Frac(12, 16)),
            PolyCommand::Close,
        ],
        intensity: BlockAlpha::Full,
        style: PolyStyle::Fill,
    },
];

pub struct ContextMenu {
    element: RefCell<Option<Vec<ComputedElement>>>,
    selected: RefCell<usize>,
    origin_x: f32,
    origin_y: f32,
    pane_id: PaneId,
    items: Vec<MenuItem>,
}

impl ContextMenu {
    pub fn new(origin_x: isize, origin_y: isize, pane_id: PaneId, can_copy: bool) -> Self {
        Self {
            element: RefCell::new(None),
            selected: RefCell::new(0),
            origin_x: origin_x.max(0) as f32,
            origin_y: origin_y.max(0) as f32,
            pane_id,
            items: vec![
                MenuItem {
                    label: "Split horizontally",
                    action: KeyAssignment::SplitHorizontal(SpawnCommand::default()),
                    enabled: true,
                },
                MenuItem {
                    label: "Split vertically",
                    action: KeyAssignment::SplitVertical(SpawnCommand::default()),
                    enabled: true,
                },
                MenuItem {
                    label: "Copy",
                    action: KeyAssignment::CopyTo(ClipboardCopyDestination::Clipboard),
                    enabled: can_copy,
                },
                MenuItem {
                    label: "Cut",
                    action: KeyAssignment::SendKey(KeyNoAction {
                        key: DeferredKeyCode::KeyCode(WindowKeyCode::Char('x')),
                        mods: WindowModifiers::CTRL,
                    }),
                    enabled: true,
                },
                MenuItem {
                    label: "Paste",
                    action: KeyAssignment::PasteFrom(ClipboardPasteSource::Clipboard),
                    enabled: true,
                },
                MenuItem {
                    label: "Select All",
                    action: KeyAssignment::SelectAll,
                    enabled: true,
                },
                MenuItem {
                    label: "Close",
                    action: KeyAssignment::CloseCurrentPane { confirm: true },
                    enabled: true,
                },
            ],
        }
    }

    fn compute(&self, term_window: &mut TermWindow) -> anyhow::Result<Vec<ComputedElement>> {
        let font = term_window
            .fonts
            .context_menu_font()
            .expect("to resolve context menu font");
        let metrics = RenderMetrics::with_font_metrics(&font.metrics());

        let colors = &term_window.config.right_click_menu_colors;
        let panel_bg: LinearRgba = colors.background.to_linear();
        let panel_border: LinearRgba = colors.border.to_linear();
        let button_bg: LinearRgba = colors.button_background.to_linear();
        let text_color: LinearRgba = colors.foreground.to_linear();
        let disabled_text: LinearRgba = colors.disabled_foreground.to_linear();
        let selected_bg: LinearRgba = colors.hover_background.to_linear();
        let selected_border: LinearRgba = colors.focus_border.to_linear();
        let selected = *self.selected.borrow();
        let menu_width = 118.;

        let mut buttons = Vec::with_capacity(2);
        for (idx, item) in self.items.iter().take(2).enumerate() {
            let icon = if idx == 0 {
                ADD_RIGHT_ICON
            } else {
                ADD_DOWN_ICON
            };
            let bg = if idx == selected {
                selected_bg.into()
            } else {
                button_bg.into()
            };
            buttons.push(
                Element::new(
                    &font,
                    ElementContent::Poly {
                        line_width: 1,
                        poly: SizedPoly {
                            poly: icon,
                            width: Dimension::Pixels(16.),
                            height: Dimension::Pixels(16.),
                        },
                    },
                )
                .item_type(UIItemType::ContextMenuItem(idx))
                .colors(ElementColors {
                    border: BorderColor::new(if idx == selected {
                        selected_border
                    } else {
                        panel_border
                    }),
                    bg,
                    text: if item.enabled {
                        text_color.into()
                    } else {
                        disabled_text.into()
                    },
                })
                .hover_colors(Some(ElementColors {
                    border: BorderColor::new(selected_border),
                    bg: selected_bg.into(),
                    text: if item.enabled {
                        text_color.into()
                    } else {
                        disabled_text.into()
                    },
                }))
                .padding(BoxDimension {
                    left: Dimension::Pixels(15.),
                    right: Dimension::Pixels(15.),
                    top: Dimension::Pixels(7.),
                    bottom: Dimension::Pixels(7.),
                })
                .margin(BoxDimension {
                    left: Dimension::Pixels(0.),
                    right: Dimension::Pixels(if idx == 0 { 2. } else { 0. }),
                    top: Dimension::Pixels(0.),
                    bottom: Dimension::Pixels(0.),
                })
                .border(BoxDimension::new(Dimension::Pixels(1.)))
                .min_width(Some(Dimension::Pixels(16.))),
            );
        }

        let mut rows = vec![Element::new(&font, ElementContent::Children(buttons))
            .margin(BoxDimension {
                left: Dimension::Pixels(0.),
                right: Dimension::Pixels(0.),
                top: Dimension::Pixels(0.),
                bottom: Dimension::Pixels(15.),
            })
            .display(DisplayType::Block)];

        for (idx, item) in self.items.iter().enumerate().skip(2) {
            let bg = if idx == selected {
                selected_bg.into()
            } else {
                panel_bg.into()
            };
            rows.push(
                Element::new(&font, ElementContent::Text(item.label.to_string()))
                    .item_type(UIItemType::ContextMenuItem(idx))
                    .colors(ElementColors {
                        border: BorderColor::default(),
                        bg,
                        text: if item.enabled {
                            text_color.into()
                        } else {
                            disabled_text.into()
                        },
                    })
                    .hover_colors(item.enabled.then_some(ElementColors {
                        border: BorderColor::default(),
                        bg: selected_bg.into(),
                        text: text_color.into(),
                    }))
                    .padding(BoxDimension {
                        left: Dimension::Pixels(7.),
                        right: Dimension::Pixels(3.),
                        top: Dimension::Pixels(3.),
                        bottom: Dimension::Pixels(3.),
                    })
                    .min_width(Some(Dimension::Pixels(88.)))
                    .display(DisplayType::Block),
            );
        }

        let panel = Element::new(&font, ElementContent::Children(rows))
            .colors(ElementColors {
                border: BorderColor::new(panel_border),
                bg: panel_bg.into(),
                text: text_color.into(),
            })
            .padding(BoxDimension {
                left: Dimension::Pixels(10.),
                right: Dimension::Pixels(10.),
                top: Dimension::Pixels(13.),
                bottom: Dimension::Pixels(18.),
            })
            .border(BoxDimension::new(Dimension::Pixels(1.)))
            .border_corners(Some(Corners {
                top_left: SizedPoly {
                    width: Dimension::Pixels(5.),
                    height: Dimension::Pixels(5.),
                    poly: TOP_LEFT_ROUNDED_CORNER,
                },
                top_right: SizedPoly {
                    width: Dimension::Pixels(5.),
                    height: Dimension::Pixels(5.),
                    poly: TOP_RIGHT_ROUNDED_CORNER,
                },
                bottom_left: SizedPoly {
                    width: Dimension::Pixels(5.),
                    height: Dimension::Pixels(5.),
                    poly: BOTTOM_LEFT_ROUNDED_CORNER,
                },
                bottom_right: SizedPoly {
                    width: Dimension::Pixels(5.),
                    height: Dimension::Pixels(5.),
                    poly: BOTTOM_RIGHT_ROUNDED_CORNER,
                },
            }));

        let dimensions = term_window.dimensions;
        let estimated_height =
            metrics.cell_size.height as f32 * self.items.len().saturating_sub(1) as f32 + 58.;
        let x = self
            .origin_x
            .min((dimensions.pixel_width as f32 - menu_width - 12.).max(8.));
        let y = self
            .origin_y
            .min((dimensions.pixel_height as f32 - estimated_height - 12.).max(8.));

        let panel = term_window.compute_element(
            &LayoutContext {
                height: DimensionContext {
                    dpi: dimensions.dpi as f32,
                    pixel_max: dimensions.pixel_height as f32,
                    pixel_cell: metrics.cell_size.height as f32,
                },
                width: DimensionContext {
                    dpi: dimensions.dpi as f32,
                    pixel_max: dimensions.pixel_width as f32,
                    pixel_cell: metrics.cell_size.width as f32,
                },
                bounds: euclid::rect(x, y, menu_width, dimensions.pixel_height as f32 - y),
                metrics: &metrics,
                gl_state: term_window.render_state.as_ref().unwrap(),
                zindex: 110,
            },
            &panel,
        )?;

        Ok(vec![panel])
    }

    fn activate(&self, idx: usize, term_window: &mut TermWindow) -> anyhow::Result<()> {
        let action = match self.items.get(idx) {
            Some(item) if item.enabled => item.action.clone(),
            None => return Ok(()),
            Some(_) => return Ok(()),
        };
        let pane = Mux::get().get_pane(self.pane_id);
        term_window.cancel_modal();
        if let Some(pane) = pane {
            term_window.perform_key_assignment(&pane, &action)?;
        }
        Ok(())
    }

    fn move_selection(&self, delta: isize, term_window: &mut TermWindow) {
        let mut selected = self.selected.borrow_mut();
        let last = self.items.len().saturating_sub(1);
        loop {
            let next = if delta < 0 {
                selected.saturating_sub(delta.unsigned_abs())
            } else {
                selected.saturating_add(delta as usize).min(last)
            };
            if next == *selected {
                break;
            }
            *selected = next;
            if self.items[*selected].enabled {
                break;
            }
        }
        self.element.borrow_mut().take();
        term_window.invalidate_modal();
    }

    pub fn select_item(&self, idx: usize, term_window: &mut TermWindow) {
        if idx < self.items.len() && self.items[idx].enabled && *self.selected.borrow() != idx {
            *self.selected.borrow_mut() = idx;
            self.element.borrow_mut().take();
            term_window.invalidate_modal();
        }
    }
}

impl Modal for ContextMenu {
    fn mouse_event(&self, _event: MouseEvent, _term_window: &mut TermWindow) -> anyhow::Result<()> {
        Ok(())
    }

    fn key_down(
        &self,
        key: KeyCode,
        mods: KeyModifiers,
        term_window: &mut TermWindow,
    ) -> anyhow::Result<bool> {
        match (key, mods) {
            (KeyCode::Escape, KeyModifiers::NONE) => term_window.cancel_modal(),
            (KeyCode::UpArrow, KeyModifiers::NONE) => self.move_selection(-1, term_window),
            (KeyCode::DownArrow, KeyModifiers::NONE) => self.move_selection(1, term_window),
            (KeyCode::Enter, KeyModifiers::NONE) => {
                self.activate(*self.selected.borrow(), term_window)?
            }
            _ => return Ok(false),
        }
        Ok(true)
    }

    fn computed_element(
        &self,
        term_window: &mut TermWindow,
    ) -> anyhow::Result<Ref<'_, [ComputedElement]>> {
        if self.element.borrow().is_none() {
            self.element
                .borrow_mut()
                .replace(self.compute(term_window)?);
        }
        Ok(Ref::map(self.element.borrow(), |value| {
            value.as_ref().unwrap().as_slice()
        }))
    }

    fn reconfigure(&self, _term_window: &mut TermWindow) {
        self.element.borrow_mut().take();
    }
}

impl TermWindow {
    pub fn show_context_menu(&mut self, x: isize, y: isize, pane_id: PaneId) {
        let can_copy = Mux::get()
            .get_pane(pane_id)
            .map(|pane| !self.selection_text(&pane).is_empty())
            .unwrap_or(false);
        self.set_modal(std::rc::Rc::new(ContextMenu::new(x, y, pane_id, can_copy)));
    }

    pub fn context_menu_is_open(&self) -> bool {
        self.get_modal()
            .map(|modal| modal.downcast_ref::<ContextMenu>().is_some())
            .unwrap_or(false)
    }

    pub fn hover_context_menu_item(&mut self, idx: usize) {
        let modal = self.get_modal();
        if let Some(menu) = modal
            .as_ref()
            .and_then(|modal| modal.downcast_ref::<ContextMenu>())
        {
            menu.select_item(idx, self);
        }
    }

    pub fn activate_context_menu_item(&mut self, idx: usize) {
        let modal = self.get_modal();
        if let Some(menu) = modal
            .as_ref()
            .and_then(|modal| modal.downcast_ref::<ContextMenu>())
        {
            if let Err(err) = menu.activate(idx, self) {
                log::error!("failed to activate context menu item: {err:#}");
            }
        }
    }
}
