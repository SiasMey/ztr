use handlebars::Handlebars;
use rand::Rng;
use std::io;
use std::io::Write;
use std::iter;
use std::{error, fs, path};
use ztr::{NewNote, Note};

fn open_create(
    zk_root: &path::Path,
    name_generator: &dyn Fn() -> String,
    note: &NewNote,
    default: &ztr::DefaultNote,
) -> io::Result<path::PathBuf> {
    let note_name = name_generator() + ".md";
    let note_path = zk_root.join(&note_name);
    let mut file = fs::File::create(&note_path)?;

    let resolved_note = resolve_note_defaults(&note_name, note, default);

    let content = render_note_template(&resolved_note).unwrap_or(String::from(""));
    write!(file, "{}", &content)?;

    Ok(note_path.to_path_buf())
}

fn resolve_note_defaults(name: &str, note: &NewNote, default: &ztr::DefaultNote) -> Note {
    Note {
        template: note.template.clone().unwrap_or(default.template.clone()),
        filename: name.to_string(),
        title: note.title.clone().unwrap_or(default.title.clone()),
        content: note.content.clone().unwrap_or(default.content.clone()),
        tags: note.tags.clone().unwrap_or(default.tags.clone()),
    }
}

fn render_note_template(note: &Note) -> Result<String, Box<dyn error::Error>> {
    let mut hb = Handlebars::new();
    hb.register_template_string("note", &note.template)?;

    let output = hb.render("note", &note).unwrap().to_string();

    Ok(output)
}

fn generate() -> String {
    let len = 10;
    const CHARSET: &[u8] = b"abcdefghijklmnopqrstuvwxyz0123456789";
    let mut rng = rand::thread_rng();
    let one_char = || CHARSET[rng.gen_range(0..CHARSET.len())] as char;
    iter::repeat_with(one_char).take(len).collect()
}

pub mod ztr {
    use serde_derive::Serialize;

    use super::*;
    use std::path;

    pub struct NewNote {
        pub template: Option<String>,
        pub title: Option<String>,
        pub content: Option<String>,
        pub tags: Option<Vec<String>>,
    }

    impl NewNote {
        pub fn new(
            template: Option<String>,
            title: Option<String>,
            content: Option<String>,
            tags: Option<Vec<String>>,
        ) -> Self {
            Self {
                template,
                title,
                content,
                tags,
            }
        }
    }

    #[derive(Serialize)]
    pub struct Note {
        pub template: String,
        pub filename: String,
        pub title: String,
        pub content: String,
        pub tags: Vec<String>,
    }

    pub struct DefaultNote {
        pub template: String,
        pub title: String,
        pub content: String,
        pub tags: Vec<String>,
    }

    impl DefaultNote {
        pub fn new() -> Self {
            Self {
                template: String::from(
                    "# {{title}}\n\n{{content}}\n\n{{#if tags}}{{#each tags}}#[[{{this}}]] {{/each}}{{/if}}"
                ),
                title: String::from("New zettle"),
                content: String::from("New note content"),
                tags: vec!["fleeting".to_string()]
            }
        }
    }

    impl Default for DefaultNote {
        fn default() -> Self {
            Self::new()
        }
    }

    pub fn create(zk_root: &path::Path) -> path::PathBuf {
        open_create(
            zk_root,
            &(generate),
            &(NewNote {
                template: None,
                title: None,
                content: None,
                tags: None,
            }),
            &DefaultNote::new(),
        )
        .unwrap()
    }
}

#[cfg(test)]
mod tests {
    use crate::ztr::DefaultNote;

    use super::*;
    use assert_fs::prelude::*;

    fn create_note_defaults() -> DefaultNote {
        DefaultNote {
            template: String::from(
                "{{#if tags}}---\ntags:\n{{#each tags}}  - {{this}}\n{{/each}}\n---\n\n{{/if}}# {{title}}\n\n{{content}}"
            ),
            title: String::from(""),
            content: String::from(""),
            tags: vec!(),
        }
    }

    #[test]
    fn test_create_should_return_note_path() {
        let temp_dir = assert_fs::TempDir::new().unwrap();

        let zk_root = temp_dir.path().to_path_buf();
        let name_generator = || String::from("test");
        let note = NewNote::new(None, None, None, None);
        let default = create_note_defaults();

        let result = open_create(&zk_root, &name_generator, &note, &default);

        assert_eq!(result.unwrap(), temp_dir.path().join("test.md"));
    }

    #[test]
    fn test_create_should_write_new_note() {
        let temp_dir = assert_fs::TempDir::new().unwrap();

        let zk_root = temp_dir.path().to_path_buf();
        let name_generator = || String::from("test");
        let note = NewNote::new(None, None, None, None);
        let default = create_note_defaults();

        let _result = open_create(&zk_root, &name_generator, &note, &default);

        assert!(temp_dir
            .child("test.md")
            .try_exists()
            .expect("Failed to check if file 'test.md' exists in temp dir"));
    }

    #[test]
    fn test_create_should_populate_title_in_template() {
        let temp_dir = assert_fs::TempDir::new().unwrap();

        let zk_root = temp_dir.path().to_path_buf();
        let name_generator = || String::from("test");
        let note = NewNote::new(
            Some(String::from("{{title}}")),
            Some(String::from("test-title")),
            None,
            None,
        );
        let default = create_note_defaults();

        let result = open_create(&zk_root, &name_generator, &note, &default);

        let output = std::fs::read_to_string(result.unwrap()).unwrap();
        assert_eq!(output, "test-title");
    }

    #[test]
    fn test_create_should_populate_content_in_template() {
        let temp_dir = assert_fs::TempDir::new().unwrap();

        let zk_root = temp_dir.path().to_path_buf();
        let name_generator = || String::from("test");
        let note = NewNote::new(
            Some(String::from("{{content}}")),
            None,
            Some(String::from("test-content")),
            None,
        );
        let default = create_note_defaults();

        let result = open_create(&zk_root, &name_generator, &note, &default);

        let output = std::fs::read_to_string(result.unwrap()).unwrap();
        assert_eq!(output, "test-content");
    }

    #[test]
    fn test_create_should_populate_default_template_if_none() {
        let temp_dir = assert_fs::TempDir::new().unwrap();

        let zk_root = temp_dir.path().to_path_buf();
        let name_generator = || String::from("test");
        let note = NewNote::new(
            None,
            Some(String::from("test-title")),
            Some(String::from("test-content")),
            None,
        );
        let default = create_note_defaults();

        let expected = String::from("# test-title\n\ntest-content");
        let result = open_create(&zk_root, &name_generator, &note, &default);

        let output = std::fs::read_to_string(result.unwrap()).unwrap();
        assert_eq!(output, expected);
    }

    #[test]
    fn test_create_should_populate_tags_in_supported_template() {
        let temp_dir = assert_fs::TempDir::new().unwrap();

        let zk_root = temp_dir.path().to_path_buf();
        let name_generator = || String::from("test");
        let note = NewNote::new(
            Some(String::from(
                "{{#if tags}}---\n\ntags:\n\n{{#each tags}}  -{{this}}\n{{/each}}\n---{{/if}}",
            )),
            None,
            None,
            Some(vec![String::from("test1"), String::from("test2")]),
        );
        let default = create_note_defaults();

        let expected = String::from("---\n\ntags:\n\n  -test1\n  -test2\n---");
        let result = open_create(&zk_root, &name_generator, &note, &default);

        let output = std::fs::read_to_string(result.unwrap()).unwrap();
        assert_eq!(output, expected);
    }

    #[test]
    fn test_create_should_ignore_frontmatter_without_tags_supported_template() {
        let temp_dir = assert_fs::TempDir::new().unwrap();

        let zk_root = temp_dir.path().to_path_buf();
        let name_generator = || String::from("test");
        let note = NewNote::new(
            Some(String::from(
                "{{#if tags}}---\n\ntags:\n\n{{#each tags}}  -{{this}}\n{{/each}}\n---{{/if}}",
            )),
            None,
            None,
            None,
        );
        let default = create_note_defaults();

        let expected = String::from("");
        let result = open_create(&zk_root, &name_generator, &note, &default);

        let output = std::fs::read_to_string(result.unwrap()).unwrap();
        assert_eq!(output, expected);
    }

    #[test]
    fn test_create_should_ignore_frontmatter_with_empty_tags_supported_template() {
        let temp_dir = assert_fs::TempDir::new().unwrap();

        let zk_root = temp_dir.path().to_path_buf();
        let name_generator = || String::from("test");
        let note = NewNote::new(
            Some(String::from(
                "{{#if tags}}---\n\ntags:\n\n{{#each tags}}  -{{this}}\n{{/each}}\n---{{/if}}",
            )),
            None,
            None,
            Some(vec![]),
        );
        let default = create_note_defaults();

        let expected = String::from("");
        let result = open_create(&zk_root, &name_generator, &note, &default);

        let output = std::fs::read_to_string(result.unwrap()).unwrap();
        assert_eq!(output, expected);
    }

    #[test]
    fn test_create_should_include_tags_in_default_template() {
        let temp_dir = assert_fs::TempDir::new().unwrap();

        let zk_root = temp_dir.path().to_path_buf();
        let name_generator = || String::from("test");
        let note = NewNote::new(
            None,
            None,
            None,
            Some(vec![String::from("test1"), String::from("test2")]),
        );
        let default = create_note_defaults();

        let expected = String::from("---\ntags:\n  - test1\n  - test2\n---\n\n# \n\n");
        let result = open_create(&zk_root, &name_generator, &note, &default);

        let output = std::fs::read_to_string(result.unwrap()).unwrap();
        assert_eq!(output, expected);
    }
}
