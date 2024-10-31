use iced::advanced::layout::{self, Layout};
use iced::advanced::widget::{self, tree, Tree, Widget};
use iced::advanced::{renderer, Clipboard, Shell};
use iced::theme::palette;
use iced::{
    border, event, touch, Background, Border, Color, Element, Event, Length, Rectangle, Shadow,
    Size, Theme,
};
use iced::{mouse, Padding};

use crate::common::defaults::DEFAULT_PADDING;

#[allow(missing_debug_implementations)]
pub struct SideBarItem<'a, Message, Theme = iced::Theme, Renderer = iced::Renderer>
where
    Renderer: iced::advanced::Renderer,
    Theme: Catalog,
{
    width: Length,
    height: Length,
    border_radios: f32,
    text: Element<'a, Message, Theme, Renderer>,
    padding: Padding,
    on_press: Option<OnPress<Message>>,
    class: Theme::Class<'a>,
}

impl<'a, Message, Theme, Renderer> SideBarItem<'a, Message, Theme, Renderer>
where
    Renderer: iced::advanced::Renderer,
    Theme: Catalog,
{
    pub fn border_radius(mut self, radius: usize) -> Self {
        self.border_radios = radius as f32;
        self
    }

    pub fn class(mut self, class: impl Into<Theme::Class<'a>>) -> Self {
        self.class = class.into();
        self
    }

    pub fn height(mut self, height: impl Into<Length>) -> Self {
        self.height = height.into();
        self
    }

    pub fn new(label: impl Into<Element<'a, Message, Theme, Renderer>>) -> Self {
        Self {
            width: Length::Shrink,
            height: Length::Fill,
            border_radios: 0.,
            class: Theme::default(),
            text: label.into(),
            padding: DEFAULT_PADDING,
            on_press: None,
        }
    }

    pub fn on_press(mut self, on_press: Message) -> Self {
        self.on_press = Some(OnPress::Direct(on_press));
        self
    }

    pub fn padding(mut self, padding: Padding) -> Self {
        self.padding = padding;
        self
    }

    pub fn width(mut self, width: impl Into<Length>) -> Self {
        self.width = width.into();
        self
    }
}

impl<'a, Message: 'a, Theme, Renderer> From<SideBarItem<'a, Message, Theme, Renderer>>
    for Element<'a, Message, Theme, Renderer>
where
    Theme: Catalog + 'a,
    Renderer: iced::advanced::Renderer + 'a,
    Message: std::clone::Clone,
{
    fn from(obj: SideBarItem<'a, Message, Theme, Renderer>) -> Self {
        Self::new(obj)
    }
}

impl<'a, Message, Theme, Renderer> Widget<Message, Theme, Renderer>
    for SideBarItem<'a, Message, Theme, Renderer>
where
    Renderer: renderer::Renderer,
    Theme: Catalog,
    Message: std::clone::Clone,
{
    fn children(&self) -> Vec<Tree> {
        vec![Tree::new(&self.text)]
    }

    fn diff(&self, tree: &mut Tree) {
        tree.diff_children(std::slice::from_ref(&self.text));
    }

    fn size(&self) -> Size<Length> {
        Size {
            width: self.width,
            height: Length::Shrink,
        }
    }

    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<State>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(State::default())
    }

    fn layout(
        &self,
        tree: &mut widget::Tree,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        // layout::atomic(limits, self.width, Length::Fill)
        layout::padded(limits, self.width, self.height, self.padding, |limits| {
            self.text
                .as_widget()
                .layout(&mut tree.children[0], renderer, limits)
        })
    }

    fn draw(
        &self,
        tree: &widget::Tree,
        renderer: &mut Renderer,
        theme: &Theme,
        _style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &iced::Rectangle,
    ) {
        let bounds = layout.bounds();
        let state = tree.state.downcast_ref::<State>();

        let status = if state.is_selected {
            Status::Active
        } else if cursor.is_over(bounds) {
            Status::Hovered
        } else {
            Status::Inactive
        };

        let style = theme.style(&self.class, status);

        renderer.fill_quad(
            renderer::Quad {
                bounds: layout.bounds(),
                border: border::rounded(self.border_radios),
                ..renderer::Quad::default()
            },
            style.background,
        );

        let content_layout = layout.children().next().unwrap();

        self.text.as_widget().draw(
            &tree.children[0],
            renderer,
            theme,
            &renderer::Style {
                text_color: style.text_color,
            },
            content_layout,
            cursor,
            viewport,
        );
    }

    fn mouse_interaction(
        &self,
        _tree: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        _viewport: &Rectangle,
        _renderer: &Renderer,
    ) -> mouse::Interaction {
        let is_mouse_over = cursor.is_over(layout.bounds());

        if is_mouse_over {
            mouse::Interaction::Pointer
        } else {
            mouse::Interaction::default()
        }
    }

    fn on_event(
        &mut self,
        tree: &mut Tree,
        event: Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        viewport: &Rectangle,
    ) -> event::Status {
        if let event::Status::Captured = self.text.as_widget_mut().on_event(
            &mut tree.children[0],
            event.clone(),
            layout.children().next().unwrap(),
            cursor,
            renderer,
            clipboard,
            shell,
            viewport,
        ) {
            return event::Status::Captured;
        }

        match event {
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left))
            | Event::Touch(touch::Event::FingerPressed { .. }) => {
                if self.on_press.is_some() {
                    let bounds = layout.bounds();

                    if cursor.is_over(bounds) {
                        let state = tree.state.downcast_mut::<State>();

                        state.is_selected = true;

                        let on_press = self.on_press.as_ref().map(OnPress::get).unwrap();
                        shell.publish(on_press);

                        return event::Status::Captured;
                    }
                }
            }
            _ => {}
        }

        event::Status::Ignored
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
struct State {
    is_selected: bool,
}

enum OnPress<Message> {
    Direct(Message),
}

impl<Message: Clone> OnPress<Message> {
    fn get(&self) -> Message {
        match self {
            OnPress::Direct(message) => message.clone(),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Status {
    Active,
    Inactive,
    Hovered,
    Disabled,
}

#[derive(Debug, Clone, Copy)]
pub struct Style {
    pub background: Background,
    pub text_color: Color,
}

pub trait Catalog {
    type Class<'a>;
    fn default<'a>() -> Self::Class<'a>;
    fn style(&self, item: &Self::Class<'_>, status: Status) -> Style;
}

impl Default for Style {
    fn default() -> Self {
        Self {
            background: Background::Color(Color::BLACK),
            text_color: Color::BLACK,
        }
    }
}

pub type StyleFn<'a, Theme> = Box<dyn Fn(&Theme, Status) -> Style + 'a>;

impl Catalog for Theme {
    type Class<'a> = StyleFn<'a, Self>;

    fn default<'a>() -> Self::Class<'a> {
        Box::new(load_color)
    }

    fn style(&self, class: &Self::Class<'_>, status: Status) -> Style {
        class(self, status)
    }
}

pub fn load_color(theme: &Theme, status: Status) -> Style {
    let palette = theme.extended_palette();
    let base = styled(palette.primary.strong);

    match status {
        Status::Active => styled(palette.background.base),
        Status::Inactive => styled(palette.background.weak),
        Status::Hovered => Style {
            background: Background::Color(palette.primary.base.color),
            ..styled(palette.background.strong)
        },
        Status::Disabled => disabled(base),
    }
}

fn styled(pair: palette::Pair) -> Style {
    Style {
        background: Background::Color(pair.color),
        text_color: pair.text,
    }
}

fn disabled(style: Style) -> Style {
    Style {
        background: style.background.scale_alpha(0.5),
        text_color: style.text_color.scale_alpha(0.5),
    }
}
