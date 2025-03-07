use crossbeam::channel::Sender;

use crate::{
    config::theme::WidgetThemeConfig,
    features::{component_id::YAML_KIND_DIALOG_ID, yaml::message::YamlRequest},
    logger,
    message::Message,
    ui::{
        event::EventResult,
        widget::{
            single_select::{
                FilterForm, FilterFormTheme, SelectForm, SelectFormTheme, SingleSelectTheme,
            },
            LiteralItem, SingleSelect, Widget, WidgetBase, WidgetTheme,
        },
        Window,
    },
};

pub fn kind_dialog(tx: &Sender<Message>, theme: WidgetThemeConfig) -> Widget<'static> {
    let tx = tx.clone();

    let widget_theme = WidgetTheme::from(theme.clone());
    let filter_theme = FilterFormTheme::from(theme.clone());
    let select_theme = SelectFormTheme::from(theme.clone());
    let single_select_theme = SingleSelectTheme::default().status_style(theme.list.status);

    let widget_base = WidgetBase::builder()
        .title("Kind")
        .theme(widget_theme)
        .build();

    let filter_form = FilterForm::builder().theme(filter_theme).build();

    let select_form = SelectForm::builder()
        .theme(select_theme)
        .on_select(on_select(tx))
        .build();

    SingleSelect::builder()
        .id(YAML_KIND_DIALOG_ID)
        .widget_base(widget_base)
        .filter_form(filter_form)
        .select_form(select_form)
        .theme(single_select_theme)
        .build()
        .into()
}

fn on_select(tx: Sender<Message>) -> impl Fn(&mut Window, &LiteralItem) -> EventResult {
    move |w, v| {
        logger!(info, "Select Item: {:?}", v);

        w.close_dialog();

        let Some(metadata) = v.metadata.as_ref() else {
            unreachable!()
        };

        let Some(key) = metadata.get("key") else {
            unreachable!()
        };

        let Ok(kind) = serde_json::from_str(key) else {
            unreachable!()
        };

        tx.send(YamlRequest::Resource(kind).into())
            .expect("Failed to send YamlRequest::Resource");

        EventResult::Nop
    }
}
