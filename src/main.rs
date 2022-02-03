use std::collections::HashMap;
use std::result::Result::Err;
use std::io::prelude::*;
use std::io::BufWriter;

// base traits
trait Node {
    fn write(&self, stream: &mut BufWriter<Box<dyn std::io::Write>>) -> std::io::Result<()>;
}

// text DOM nodes
struct Text {
    text: String
}
impl Node for Text {
    fn write(&self, stream: &mut BufWriter<Box<dyn std::io::Write>>) -> std::io::Result<()> {
        stream.write_all(self.text.as_bytes())?;
        
        Ok(())
    }
}

// tag DOM nodes
struct Tag {
    tag_name: String,
    children: Vec<Box<dyn Node>>,
    properties: HashMap<String, String>
}
impl Tag {
    fn new(tag_name: &str) -> Tag {
        Tag {tag_name: tag_name.to_string(), children: Vec::new(), properties: HashMap::new()}
    }
}
impl Node for Tag {
    fn write(&self, stream: &mut BufWriter<Box<dyn std::io::Write>>) -> std::io::Result<()> {
        stream.write_all(b"<")?;
        
        if self.tag_name == "smc" {
            stream.write_all(b"div")?;
        } else {
            stream.write_all(self.tag_name.as_bytes())?;
        }
        
        for (prop_name, prop_val) in &self.properties {
            stream.write_all(b" ")?;
            
            stream.write_all(prop_name.as_bytes())?;
            stream.write_all(b"=\"")?;
            stream.write_all(prop_val.as_bytes())?;
            stream.write_all(b"\"")?;
        }
        
        stream.write_all(b">")?;
        
        if self.tag_name == "smc" {
            let writer: Box<dyn std::io::Write> = Box::new(std::io::Cursor::new(Vec::new()));
            let mut buf: BufWriter<Box<dyn std::io::Write>> = BufWriter::new(writer);
            
            for child_node in &self.children {
                child_node.write(&mut buf)?;
            }
            
            let writer: Box<dyn std::io::Write> = match buf.into_inner() {
                Ok(a) => a,
                Err(_) => {panic!("writer not found")}
            };
            
            let writer_dptr: *const Box<dyn std::io::Write> = &writer;
            let writer_cptr: *const Box<std::io::Cursor<Vec<u8>>> = writer_dptr as *const Box<std::io::Cursor<Vec<u8>>>;
            
            let writer_a: &Box<std::io::Cursor<Vec<u8>>> = unsafe {
                 &(*writer_cptr)
            };
            
            let data: &Vec<u8> = writer_a.get_ref();
            let s: &str = std::str::from_utf8(data).unwrap();
            
            stream.write_all(s.replace("\0", self.children.len().to_string().as_str()).as_bytes())?;
        } else {
            for child_node in &self.children {
                child_node.write(stream)?;
            }
        }
        
        stream.write_all(b"</")?;
        if self.tag_name == "smc" {
            stream.write_all(b"div")?;
        } else {
            stream.write_all(self.tag_name.as_bytes())?;
        }
        stream.write_all(b">")?;
        
        Ok(())
    }
}

fn boxed_text(text: &str) -> Box<Text> {
    Box::new(Text {text: text.to_string()})
}

#[allow(dead_code)]
fn boxed_page(head_nodes: Vec<Box<dyn Node>>, body_nodes: Vec<Box<dyn Node>>) -> Box<Tag> {
    let head = Box::new(Tag {tag_name: "head".to_string(), children: head_nodes, properties: HashMap::new()});
    let body = Box::new(Tag {tag_name: "body".to_string(), children: body_nodes, properties: HashMap::new()});
    
    Box::new(Tag {tag_name: "html".to_string(), children: vec![head, body], properties: HashMap::new()})
}

fn process_template(tag_name: String) -> Tag {
    match tag_name.as_str() {
        "!Document" => Tag {tag_name: "html".to_string(), properties: HashMap::new(), children: vec![boxed_text("<meta charset=\"utf-8\">")]},
        "!Rocyonery" => Tag {
            tag_name: "div".to_string(),
            properties: HashMap::from([
                ("style".to_string(), "background-color:#fdd;width:100%;height:60px;line-height:60px;text-align:center;".to_string())
            ]),
            children: vec![boxed_text("Создано с помощью дешаблонизатора от Rocyonery, Inc.")]
        },
        "!Fullwidth" => Tag {
            tag_name: "div".to_string(),
            properties: HashMap::from([
                ("style".to_string(), "width: 100%;".to_string())
            ]),
            children: Vec::new()
        },
        "!Title" => Tag {
            tag_name: "div".to_string(),
            properties: HashMap::from([
                ("style".to_string(), "width: 100%; height: 60px; line-height: 60px; font-size: 20px; text-align: center;".to_string())
            ]),
            children: Vec::new()
        },
        "!Subtitle" => Tag {
            tag_name: "div".to_string(),
            properties: HashMap::from([
                ("style".to_string(), "width: 100%; font-size: 16px; text-align: center;".to_string())
            ]),
            children: Vec::new()
        },
        "!SmartColumns" => Tag {
            tag_name: "smc".to_string(),
            properties: HashMap::from([
                ("style".to_string(), "width: 100%;".to_string())
            ]),
            children: Vec::new()
        },
        "!Column" => Tag {
            tag_name: "div".to_string(),
            properties: HashMap::from([
                ("style".to_string(), "width: calc(100% / \0 - 6px);display:inline-block;vertical-align:top;margin:3px;".to_string())
            ]),
            children: Vec::new()
        },
        _ => Tag::new(tag_name.as_str())
    }
}

fn process_style_template(prop_name: &str, prop_val: &str, node: &mut Box<Tag>, global_keys: &mut HashMap<String, String>) {
    let mut prop_val: String = prop_val.to_string();
    
    for (l, r) in global_keys.iter() {
        if prop_val.contains(l) {
            prop_val = prop_val.replace(l, r);
        }
    }
    
    let styles: &mut String = node.properties.entry("style".to_string()).or_insert(String::new());
    match prop_name {
        "Set" => {
            let (l, r) = prop_val.split_once(' ').unwrap();
            global_keys.insert(l.to_string(), r.to_string());
            return;
        },
        "Width" => {
            styles.push_str("width: ");
            styles.push_str(&prop_val);
        },
        "Height" => {
            styles.push_str("height: ");
            styles.push_str(&prop_val);
        },
        "LHeight" => {
            styles.push_str("line-height: ");
            styles.push_str(&prop_val);
        },
        "Text_size" => {
            styles.push_str("font-size: ");
            styles.push_str(&prop_val);
        },
        "Text_type" => {
            styles.push_str("font-family: ");
            styles.push_str(&prop_val);
        },
        "Back_fill" => {
            styles.push_str("background-color: ");
            styles.push_str(&prop_val);
        },
        "Centred" => {
            styles.push_str("text-align: center");
        },
        "Back_grad" => {
            styles.push_str("background: linear-gradient(");
            styles.push_str(&prop_val);
            styles.push_str(")");
        },
        _ => {
            println!("unprocessed style: {}:{}", prop_name, prop_val);
            return;
        }
    };
    styles.push(';');
}

fn read_kv(kv_file: &str) -> Result<Box<dyn Node>, String> {
    let kv_data: String = match std::fs::read_to_string(kv_file) {
        Ok(s) => s,
        Err(e) => {return Err(format!("I/O error when reading file: {:?}", e))}
    };
    
    // let mut nodes_by_indent: Vec<(usize, Box<dyn Node>)> = Vec::new();
    // we can't match whether Node is Tag
    // so we store only Tags
    
    let mut root_tag: Box<Tag> = Box::new(Tag::new(""));
    let mut nodes_by_indent: Vec<(isize, &mut Box<Tag>)> = vec![(-1, &mut root_tag)];
    
    let mut global_keys: HashMap<String, String> = HashMap::new();
    
    for line in kv_data.lines() {
        let line_rtrim: &str = line.trim_end();
        let line_proc: &str = line_rtrim.trim_start();
        
        if line_proc.len() == 0 {continue;}
        if line_proc.starts_with('#') {continue;}
        
        let indent: isize = (line_rtrim.len() - line_proc.len()).try_into().unwrap();
        
        if line_proc.starts_with('@') {
            match line_proc.find(':') {
                Some(n) => {
                    let (b, c) = line_proc.split_at(n + 1);
                    let b = b.strip_suffix(':').unwrap().strip_prefix('@').unwrap();
                    let c = c.trim();
                    
                    if nodes_by_indent.len() <= 1 {
                        return Err(format!("root node cannot be a property"));
                    } else {
                        let node: &mut Box<Tag> = nodes_by_indent.last_mut().unwrap().1;
                        process_style_template(b, c, node, &mut global_keys);
                        continue;
                    }
                },
                None => {
                    return Err(format!("not found : in property: {}", line_proc));
                }
            };
        }
        
        if line_proc.ends_with(':') {
            let mut tag_name: String = line_proc.to_string();
            tag_name.pop();
            
            let node: Box<Tag> = Box::new(process_template(tag_name));
            
            while nodes_by_indent.len() > 0 && nodes_by_indent.last().unwrap().0 >= indent {
                nodes_by_indent.pop();
            }
            
            let parent_node: &mut Box<Tag> = nodes_by_indent.last_mut().unwrap().1;
            
            parent_node.children.push(node);
            
            let self_node: &mut Box<dyn Node> = parent_node.children.last_mut().unwrap();
            let self_pnode: *mut Box<dyn Node> = self_node;
            
            // this cast is really safe because we've just pushed our Tag to parent_node.children
            // so we know it remains a Tag
            let self_unode: *mut Box<Tag> = self_pnode as *mut Box<Tag>;
            
            unsafe {
                let self_tag: &mut Box<Tag> = &mut *self_unode;
                nodes_by_indent.push((indent, self_tag));
            }
        } else if line_proc.contains(':') {
            let (l, r) = line_proc.split_once(':').unwrap();
            let l = l.trim();
            let r = r.trim();
            
            if nodes_by_indent.len() <= 1 {
                return Err(format!("root node cannot be a property"));
            } else {
                let node: &mut Box<Tag> = nodes_by_indent.last_mut().unwrap().1;
                node.properties.entry(l.to_string()).or_insert(String::new()).push_str(r);
            }
        } else {
            let node: Box<Text> = boxed_text(line_proc);
            
            while nodes_by_indent.len() > 0 && nodes_by_indent.last().unwrap().0 >= indent {
                nodes_by_indent.pop();
            }
            
            if indent == 0 || nodes_by_indent.len() == 0 {
                return Err("invalid KV structure: text is root node".to_string());
            }
            
            nodes_by_indent.last_mut().unwrap().1.children.push(node);
            // nodes_by_indent.push((indent, node)); // text can't be parent of any node
        }
    }
    
    drop(nodes_by_indent);
    
    match root_tag.children.len() {
        0 => Err("empty document".to_string()),
        _ => Ok(root_tag.children.swap_remove(0))
    }
}

fn main() {
    let serving_page = read_kv("main.kv").unwrap();
    
    let stdout: Box<dyn std::io::Write> = Box::new(std::io::stdout());
    let mut writer = std::io::BufWriter::new(stdout);
    
    serving_page.write(&mut writer).unwrap();
}
