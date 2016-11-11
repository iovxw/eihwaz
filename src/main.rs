extern crate gtk;
extern crate glib;
extern crate serde_json;

mod config;

use std::cell::RefCell;

use gtk::prelude::*;
use gtk::{
    CellRendererText, ListStore, TreeView, TreeViewColumn,
    Window, WindowType
};

#[derive(Clone, Debug, PartialEq)]
pub enum ItemValue {
    File(String),
    Command(String),
    Application(String),
    Index(Vec<Item>),
}

#[derive(Clone, Debug, PartialEq)]
pub struct Item {
    key: char,
    text: String,
    value: ItemValue,
}

fn create_and_fill_model(entries: &[Item]) -> ListStore {
    let model = ListStore::new(&[String::static_type(), String::static_type()]);

    for entry in entries.iter() {
        model.insert_with_values(None, &[0, 1], &[&entry.key.to_string(), &entry.text]);
    }
    model
}

fn append_column(tree: &TreeView, id: i32) {
    let column = TreeViewColumn::new();
    let cell = CellRendererText::new();

    column.pack_start(&cell, true);
    column.add_attribute(&cell, "text", id);
    tree.append_column(&column);
}

fn create_and_setup_view() -> TreeView {
    let tree = TreeView::new();

    tree.set_headers_visible(false);
    append_column(&tree, 0);
    append_column(&tree, 1);
    tree
}

fn main() {
    if gtk::init().is_err() {
        println!("Failed to initialize GTK.");
        return;
    }

    let window = Window::new(WindowType::Toplevel);
    window.set_default_size(350, 70);
    window.set_keep_above(true);
    window.set_skip_taskbar_hint(true);
    window.set_decorated(false);

    window.connect_focus_out_event(|_, _| {
        gtk::main_quit();
        Inhibit(false)
    });
    window.connect_delete_event(|_, _| {
        gtk::main_quit();
        Inhibit(false)
    });

    let tree = create_and_setup_view();
    tree.set_enable_search(false);

    let data = config::load_config().unwrap();
    let model = create_and_fill_model(&data);

    tree.set_model(Some(&model));

    let index = RefCell::new(data);
    tree.connect_key_press_event(move |tree_view, key| {
        let mut keyval = key.get_keyval();

        if keyval == 65293 { // enter
            let selection = tree_view.get_selection();
            if let Some((model, iter)) = selection.get_selected() {
                keyval = model.get_value(&iter, 0).get::<String>().unwrap().pop().unwrap() as u32;
            }
        }

        let mut index_swap:Option<Vec<Item>> = None;
        for d in index.borrow().iter() {
            if d.key as u32 == keyval {
                println!("{} {}", key.get_keyval(), d.key);
                match d.value {
                    ItemValue::Command(ref cmd) => {
                        println!("{}", cmd);
                        gtk::main_quit();
                    },
                    ItemValue::Index(ref new_index) => {
                        tree_view.set_model(Some(&create_and_fill_model(&new_index)));
                        index_swap = Some((*new_index).clone());
                    },
                    _ => (),
                }
                break;
            }
        }
        if let Some(new_index) = index_swap {
            *index.borrow_mut() = new_index;
        }
        Inhibit(false)
    });

    window.add(&tree);

    window.show_all();
    gtk::main();
}
