use tera::{Context, Tera, try_get_value};
use serde_json::value::{to_value, Value};
use std::collections::HashMap;
use crate::domain::NewSubscriber;

lazy_static::lazy_static! {
    pub static ref TEMPLATES: Tera = {
        let mut tera = match Tera::new("templates/**/*") {
            Ok(t) => t,
            Err(e) => {
                println!("Parsing error(s): {}", e);
                ::std::process::exit(1);
            }
        };
        tera.autoescape_on(vec!["html", ".sql"]);
        tera.register_filter("do_nothing", do_nothing_filter);
        tera
    };
}

pub fn do_nothing_filter(value: &Value, _: &HashMap<String, Value>) -> Result<Value, tera::Error> {
    let s = try_get_value!("do_nothing_filter", "value", String, value);
    Ok(to_value(s).unwrap())
}

pub fn generate_html_template(
    new_subscriber: &NewSubscriber,
    url: &str
) -> Result<String, tera::Error> {
    let mut context = Context::new();
    context.insert("username", new_subscriber.name.as_ref());
    context.insert("url", url);

    Ok(TEMPLATES.render("index.html", &context)?)
}

#[cfg(test)]
mod tests {
    use crate::domain::SubscriberName;
    use crate::domain::SubscriberEmail;
    use crate::domain::NewSubscriber;

    #[test]
    fn create_an_html_template() {

        let new_subscriber = NewSubscriber {
            email: SubscriberEmail::parse("ramses.hdz30@gmail.com".to_string()).unwrap(),
            name: SubscriberName::parse("Ramses".to_string()).unwrap()
        };

        match super::generate_html_template(&new_subscriber, "https://google.com") {
            Ok(html) => println!("{}", html),
            Err(err) => println!("Error: {}", err)
        }
    }
}
