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
use mux::pane::Pane;
use std::cell::{Cell, Ref, RefCell};
use std::sync::Arc;
use wezterm_dynamic::Value;
use wezterm_term::{KeyCode, KeyModifiers, MouseEvent};
use window::color::LinearRgba;
use window::{KeyCode as WindowKeyCode, Modifiers as WindowModifiers};

struct MenuItem {
    label: &'static str,
    action: MenuAction,
}

#[derive(Clone)]
enum MenuAction {
    Key(KeyAssignment),
    Themes,
    Back,
    ColorScheme(&'static str),
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
    in_themes: Cell<bool>,
    origin_x: f32,
    origin_y: f32,
    pane: Arc<dyn Pane>,
    main_items: Vec<MenuItem>,
    theme_items: Vec<MenuItem>,
}

impl ContextMenu {
    pub fn new(origin_x: isize, origin_y: isize, pane: Arc<dyn Pane>) -> Self {
        Self {
            element: RefCell::new(None),
            selected: RefCell::new(0),
            in_themes: Cell::new(false),
            origin_x: origin_x.max(0) as f32,
            origin_y: origin_y.max(0) as f32,
            pane,
            main_items: vec![
                MenuItem {
                    label: "Split horizontally",
                    action: MenuAction::Key(
                        KeyAssignment::SplitHorizontal(SpawnCommand::default()),
                    ),
                },
                MenuItem {
                    label: "Split vertically",
                    action: MenuAction::Key(KeyAssignment::SplitVertical(SpawnCommand::default())),
                },
                MenuItem {
                    label: "Copy",
                    action: MenuAction::Key(KeyAssignment::CopyTo(
                        ClipboardCopyDestination::Clipboard,
                    )),
                },
                MenuItem {
                    label: "Cut",
                    action: MenuAction::Key(KeyAssignment::SendKey(KeyNoAction {
                        key: DeferredKeyCode::KeyCode(WindowKeyCode::Char('x')),
                        mods: WindowModifiers::CTRL,
                    })),
                },
                MenuItem {
                    label: "Paste",
                    action: MenuAction::Key(KeyAssignment::PasteFrom(
                        ClipboardPasteSource::Clipboard,
                    )),
                },
                MenuItem {
                    label: "Select All",
                    action: MenuAction::Key(KeyAssignment::SelectAll),
                },
                MenuItem {
                    label: "Themes >",
                    action: MenuAction::Themes,
                },
                MenuItem {
                    label: "Close",
                    action: MenuAction::Key(KeyAssignment::CloseCurrentPane { confirm: false }),
                },
            ],
            theme_items: vec![
                MenuItem {
                    label: "< Back",
                    action: MenuAction::Back,
                },
                MenuItem {
                    label: config::TERMINATOR_GRUVBOX_SOLARIZED,
                    action: MenuAction::ColorScheme(config::TERMINATOR_GRUVBOX_SOLARIZED),
                },
                MenuItem {
                    label: "Gruvbox Dark Soft",
                    action: MenuAction::ColorScheme("Gruvbox dark, soft (base16)"),
                },
                MenuItem {
                    label: "Everforest Dark Soft",
                    action: MenuAction::ColorScheme("Everforest Dark Soft (Gogh)"),
                },
                MenuItem {
                    label: "Nord",
                    action: MenuAction::ColorScheme("Nord (Gogh)"),
                },
                MenuItem {
                    label: "Solarized Dark",
                    action: MenuAction::ColorScheme("Solarized Dark (Gogh)"),
                },
            ],
        }
    }

    fn items(&self) -> &[MenuItem] {
        if self.in_themes.get() {
            &self.theme_items
        } else {
            &self.main_items
        }
    }

    fn show_themes(&self, show: bool, term_window: &mut TermWindow) {
        self.in_themes.set(show);
        *self.selected.borrow_mut() = if show { 0 } else { self.main_items.len() - 2 };
        self.element.borrow_mut().take();
        term_window.invalidate_modal();
    }

    fn themes_is_selected(&self) -> bool {
        let selected = *self.selected.borrow();
        matches!(&self.items()[selected].action, MenuAction::Themes)
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
        let selected_bg: LinearRgba = colors.hover_background.to_linear();
        let selected_border: LinearRgba = colors.focus_border.to_linear();
        let selected = *self.selected.borrow();
        let in_themes = self.in_themes.get();
        let items = self.items();
        let menu_width = if in_themes { 235. } else { 118. };

        let mut buttons = Vec::with_capacity(2);
        for idx in 0..if in_themes { 0 } else { 2 } {
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
                    text: text_color.into(),
                })
                .hover_colors(Some(ElementColors {
                    border: BorderColor::new(selected_border),
                    bg: selected_bg.into(),
                    text: text_color.into(),
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

        let mut rows = vec![];
        if !in_themes {
            rows.push(
                Element::new(&font, ElementContent::Children(buttons))
                    .margin(BoxDimension {
                        left: Dimension::Pixels(0.),
                        right: Dimension::Pixels(0.),
                        top: Dimension::Pixels(0.),
                        bottom: Dimension::Pixels(15.),
                    })
                    .display(DisplayType::Block),
            );
        }

        for (idx, item) in items.iter().enumerate().skip(if in_themes { 0 } else { 2 }) {
            let bg = if idx == selected {
                selected_bg.into()
            } else {
                panel_bg.into()
            };
            let label = match &item.action {
                MenuAction::ColorScheme(name)
                    if term_window.config.color_scheme.as_deref() == Some(name) =>
                {
                    format!("* {}", item.label)
                }
                _ => item.label.to_string(),
            };
            rows.push(
                Element::new(&font, ElementContent::Text(label))
                    .item_type(UIItemType::ContextMenuItem(idx))
                    .colors(ElementColors {
                        border: BorderColor::default(),
                        bg,
                        text: text_color.into(),
                    })
                    .hover_colors(Some(ElementColors {
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
        let estimated_height = metrics.cell_size.height as f32
            * items.len().saturating_sub(if in_themes { 0 } else { 1 }) as f32
            + 58.;
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
        let Some(item) = self.items().get(idx) else {
            return Ok(());
        };
        let action = item.action.clone();
        match action {
            MenuAction::Themes => self.show_themes(true, term_window),
            MenuAction::Back => self.show_themes(false, term_window),
            MenuAction::ColorScheme(name) => {
                let mut overrides = match &term_window.config_overrides {
                    Value::Object(object) => object.clone(),
                    _ => Default::default(),
                };
                overrides.insert(
                    Value::String("color_scheme".to_string()),
                    Value::String(name.to_string()),
                );
                term_window.cancel_modal();
                term_window.config_overrides = Value::Object(overrides);
                term_window.config_was_reloaded();
            }
            MenuAction::Key(action) => {
                term_window.cancel_modal();
                match action {
                    KeyAssignment::CloseCurrentPane { confirm } => {
                        term_window.close_pane(&self.pane, confirm)
                    }
                    KeyAssignment::CopyTo(destination) => {
                        let text = term_window.selection_text(&self.pane);
                        if !text.is_empty() {
                            term_window.copy_to_clipboard(destination, text);
                        }
                    }
                    _ => {
                        term_window.perform_key_assignment(&self.pane, &action)?;
                    }
                }
            }
        }
        Ok(())
    }

    fn move_selection(&self, delta: isize, term_window: &mut TermWindow) {
        let mut selected = self.selected.borrow_mut();
        let last = self.items().len().saturating_sub(1);
        *selected = if delta < 0 {
            selected.saturating_sub(delta.unsigned_abs())
        } else {
            selected.saturating_add(delta as usize).min(last)
        };
        self.element.borrow_mut().take();
        term_window.invalidate_modal();
    }

    pub fn select_item(&self, idx: usize, term_window: &mut TermWindow) {
        if idx < self.items().len() && *self.selected.borrow() != idx {
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
            (KeyCode::Escape | KeyCode::LeftArrow, KeyModifiers::NONE) if self.in_themes.get() => {
                self.show_themes(false, term_window)
            }
            (KeyCode::Escape, KeyModifiers::NONE) => term_window.cancel_modal(),
            (KeyCode::UpArrow, KeyModifiers::NONE) => self.move_selection(-1, term_window),
            (KeyCode::DownArrow, KeyModifiers::NONE) => self.move_selection(1, term_window),
            (KeyCode::RightArrow, KeyModifiers::NONE) if self.themes_is_selected() => {
                self.show_themes(true, term_window)
            }
            (KeyCode::Enter, KeyModifiers::NONE) => {
                let selected = *self.selected.borrow();
                self.activate(selected, term_window)?
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
    pub fn show_context_menu(&mut self, x: isize, y: isize, pane: Arc<dyn Pane>) {
        self.set_modal(std::rc::Rc::new(ContextMenu::new(x, y, pane)));
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
