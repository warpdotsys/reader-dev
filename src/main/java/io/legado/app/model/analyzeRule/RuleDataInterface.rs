// package io.legado.app.model.analyzeRule

trait RuleDataInterface {
    fn variable_map(&self) -> &HashMap<String, String>;

    fn get_user_name_space(&self) -> String;

    fn put_variable(&mut self, key: &str, value: Option<&str>);

    fn get_variable(&self, key: &str) -> Option<String> {
        self.variable_map().get(key).cloned()
    }
}
