use crate::{Error, error::DoomError};
use std::vec::IntoIter;

pub fn limit_content_and_see_more<'a, I>(
    limit: usize,
    components: I,
    builder: fn(IntoIter<Vec<&str>>) -> String,
    link: Option<(&str, usize)>,
) -> Result<String, Error>
where
    I: IntoIterator<Item = Vec<&'a str>> + Clone,
{
    let mut level = 0;
    let mut complete = false;
    let mut prev_message_components: Vec<Vec<&str>>;
    let mut message_components = vec![];

    let fits_in_limit = |message_variant: String| message_variant.len() <= limit;

    while !complete {
        level += 1;
        complete = true;
        prev_message_components = message_components.clone();
        message_components.clear();
        for component in components.clone() {
            let mut message_component = vec![];
            for i in 0..level {
                /* only push item if it exists */
                if i < component.len() {
                    message_component.push(component[i]);
                } else {
                    break;
                }
            }
            /* there may still be a couple of items to be appended */
            if message_component.len() == level {
                complete = false;
            }
            message_components.push(message_component);
        }

        let message = builder(message_components.clone().into_iter());
        if !fits_in_limit(message) {
            message_components = prev_message_components.clone();
            break;
        }
    }

    complete = true;

    if message_components.len() != components.clone().into_iter().count() {
        complete = false;
    }

    let mut component_iter_clone = components.clone().into_iter();
    for vec in &message_components {
        if let Some(other_vec) = component_iter_clone.next() {
            if vec.len() != other_vec.len() {
                complete = false;
            }
        } else {
            complete = false;
        }
    }

    let message = builder(message_components.clone().into_iter());
    if let Some(link) = link
        && !complete
    {
        let (url, pos) = link;
        let href = format!("[... See more]({})", url);
        let mut local_components = message_components.clone();
        let under_limit_letter_count = limit - (message.len() + href.len());
        if under_limit_letter_count > 0 {
            while local_components.len() < pos {
                local_components.push(vec![""]);
            }
            if local_components.len() == pos {
                local_components.push(vec![]);
            }
            local_components[pos].push(&*href);
        } else {
            return Err(Box::new(DoomError::NotImplementedError {
                functionality: "Handling of Insufficient Space for Link".to_string(),
            }));
        }
        Ok(builder(local_components.clone().into_iter()))
    } else {
        Ok(message)
    }
}

pub fn build_changelog(mut v: IntoIter<Vec<&str>>) -> String {
    if v.len() == 0 {
        return "".to_string();
    }
    let mut message = v.next().unwrap().join(" ").to_string();
    message = format!("{}\n\n{}", message, v.next().unwrap().join("\n\n"));
    message = format!("{}\n\n{}", message, v.next().unwrap().join(" "));
    message.to_string()
}
