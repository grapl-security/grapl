use graph_description::{DynamicNode, NodeProperty, node_property, IdStrategy, id_strategy};
use serde_json::Value;
use node::NodeT;

impl DynamicNode {
    pub fn set_property(
        &mut self,
        name: impl Into<String>,
        value: impl Into<NodeProperty>
    ) {
        self.properties.insert(
            name.into(),
            value.into().into(),
        );
    }

    pub fn set_key(&mut self, key: String) {
        self.node_key = key;
    }

    pub fn into_json(self) -> Value {
        let mut j = json!({
            "node_key": self.node_key,
            "dgraph.type": self.node_type,
            "seen_at": self.seen_at,
        });

        if let Some(asset_id) = self.asset_id {
            j["asset_id"] = asset_id.into();
        }

        for (key, prop) in self.properties {
            let prop = match prop.property {
                Some(node_property::Property::Intprop(i)) => Value::from(i),
                Some(node_property::Property::Uintprop(i)) => Value::from(i),
                Some(node_property::Property::Strprop(s)) => Value::from(s),
                None => panic!("Invalid property on DynamicNode: {}", self.node_key),
            };

            j[key] = prop;
        }

        j
    }

    pub fn get_id_strategies(&self) -> &[IdStrategy] {
        &self.id_strategy[..]
    }


    pub fn requires_asset_identification(&self) -> bool {
        for strategy in self.get_id_strategies() {
            match strategy.strategy.as_ref().unwrap() {
                id_strategy::Strategy::Session(ref strategy) => {
                    if strategy.primary_key_requires_asset_id {
                        return true
                    }
                }
                id_strategy::Strategy::Static(ref strategy) => {
                    if strategy.primary_key_requires_asset_id {
                        return true
                    }
                }
            }
        }

        false
    }

}

impl NodeT for DynamicNode {
    fn get_asset_id(&self) -> Option<&str> {
        self.asset_id.as_ref().map(String::as_str)
    }

    fn set_asset_id(&mut self, asset_id: impl Into<String>) {
        self.asset_id = Some(asset_id.into());
    }

    fn get_node_key(&self) -> &str {
        self.node_key.as_str()
    }

    fn set_node_key(&mut self, node_key: impl Into<String>) {
        self.node_key = node_key.into();
    }

    fn merge(&mut self, other: &Self) -> bool {
        if self.node_key != other.node_key {
            warn!("Attempted to merge two NetworkConnection Nodes with differing node_keys");
            return false
        }

        let mut merged = false;

        for (key, prop) in other.properties.clone() {
            let inserted = self.properties.insert(key, prop);
            if inserted.is_some() {
                merged = true;
            }
        }

        merged
    }

    fn merge_into(&mut self, other: Self) -> bool {
        if self.node_key != other.node_key {
            warn!("Attempted to merge two NetworkConnection Nodes with differing node_keys");
            return false
        }

        let mut merged = false;

        for (key, prop) in other.properties.into_iter() {
            let inserted = self.properties.insert(key, prop);
            if inserted.is_some() {
                merged = true;
            }
        }

        merged
    }
}