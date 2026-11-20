// Copyright (c) 2022 Unfolded Circle ApS, Markus Zehnder <markus.z@unfoldedcircle.com>
// SPDX-License-Identifier: MPL-2.0

//! Button entity specific logic.

use crate::client::model::EventData;
use crate::errors::ServiceError;
use serde_json::{Map, Value};
use std::collections::HashMap;
use uc_api::EntityType;
use uc_api::intg::{AvailableIntgEntity, EntityChange};

fn map_button_attributes(state: &str) -> Map<String, Value> {
    // HA scripts use on/off, buttons and scenes use a last-triggered timestamp.
    // `unknown` also means a button or scene has never been triggered, so it is
    // still available. Only HA's explicit unavailable state disables the button.
    Map::from_iter([(
        "state".into(),
        Value::from(if state == "unavailable" {
            "UNAVAILABLE"
        } else {
            "AVAILABLE"
        }),
    )])
}

pub(crate) fn button_event_to_entity_change(data: EventData) -> Result<EntityChange, ServiceError> {
    Ok(EntityChange {
        device_id: None,
        entity_type: EntityType::Button,
        entity_id: data.entity_id,
        attributes: map_button_attributes(&data.new_state.state),
    })
}

pub(crate) fn convert_button_entity(
    entity_id: String,
    state: String,
    ha_attr: &mut Map<String, Value>,
) -> Result<AvailableIntgEntity, ServiceError> {
    let friendly_name = ha_attr.get("friendly_name").and_then(|v| v.as_str());
    let name = HashMap::from([("en".into(), friendly_name.unwrap_or(&entity_id).into())]);

    Ok(AvailableIntgEntity {
        entity_id,
        device_id: None, // prepared for device_id handling
        entity_type: EntityType::Button,
        device_class: None,
        name,
        icon: None,
        description: None,
        features: None, // no optional features, default = "press"
        area: None,
        options: None,
        attributes: Some(map_button_attributes(&state)),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::client::model::EventState;
    use rstest::rstest;
    use serde_json::json;

    #[rstest]
    #[case("script.test", "off", "AVAILABLE")]
    #[case("script.test", "on", "AVAILABLE")]
    #[case("script.test", "unavailable", "UNAVAILABLE")]
    #[case("button.test", "unknown", "AVAILABLE")]
    #[case("button.test", "2026-11-20T12:00:00+00:00", "AVAILABLE")]
    #[case("button.test", "unavailable", "UNAVAILABLE")]
    #[case("input_button.test", "unknown", "AVAILABLE")]
    #[case("input_button.test", "2026-11-20T12:00:00+00:00", "AVAILABLE")]
    #[case("input_button.test", "unavailable", "UNAVAILABLE")]
    #[case("scene.test", "unknown", "AVAILABLE")]
    #[case("scene.test", "2026-11-20T12:00:00+00:00", "AVAILABLE")]
    #[case("scene.test", "unavailable", "UNAVAILABLE")]
    fn availability_in_state_refresh_and_events(
        #[case] entity_id: &str,
        #[case] state: &str,
        #[case] expected: &str,
    ) {
        let entity = convert_button_entity(
            entity_id.into(),
            state.into(),
            &mut Map::from_iter([("friendly_name".into(), json!("Test button"))]),
        )
        .unwrap();
        assert_eq!(EntityType::Button, entity.entity_type);
        assert_eq!(entity_id, entity.entity_id);
        assert_eq!("Test button", entity.name["en"]);
        // The controller forwards these attributes in the entity_states response
        // when the Remote refreshes its state after waking up.
        assert_eq!(
            json!({"state": expected}),
            json!(entity.attributes.unwrap())
        );

        let change = button_event_to_entity_change(EventData {
            entity_id: entity_id.into(),
            new_state: EventState {
                state: state.into(),
                attributes: None,
            },
        })
        .unwrap();
        assert_eq!(EntityType::Button, change.entity_type);
        assert_eq!(entity_id, change.entity_id);
        assert_eq!(json!({"state": expected}), json!(change.attributes));
    }
}
