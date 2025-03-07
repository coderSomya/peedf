use std::process::exit;
use std::fs::File;
use std::io::{Write, BufWriter};
use lopdf::{Document, Object};

fn main() {
    let pdf_path = "paper.pdf";
    let output_path = "paper.txt";
    
    let pdf = Document::load(&pdf_path).map_err(|err| {
        eprintln!("Error loading PDF: {}", err);
        exit(1);
    }).unwrap();

    // Create the output file with UTF-8 encoding
    let output_file = File::create(output_path).map_err(|err| {
        eprintln!("Error creating output file: {}", err);
        exit(1);
    }).unwrap();
    
    let mut writer = BufWriter::new(output_file);
    
    writer.write_all(&[0xEF, 0xBB, 0xBF]).unwrap();
    
    let pages = pdf.get_pages().len();
    writeln!(writer, "Number of pages: {}", pages).unwrap();
    
    for (page_num, object_id) in pdf.get_pages() {
        writeln!(writer, "==== Page {} ====", page_num).unwrap();
        
        // Extract text from the page
        if let Ok(content) = pdf.get_page_content(object_id) {
            if let Ok(text) = extract_text(&content) {
                writeln!(writer, "{}", text).unwrap();
            } else {
                writeln!(writer, "Failed to extract text from page {}", page_num).unwrap();
            }
        } else {
            writeln!(writer, "Could not get content for page {}", page_num).unwrap();
        }
    }
    
    writer.flush().unwrap();
    
    println!("Text extraction complete. Output saved to {} in UTF-8 format", output_path);
}

fn extract_text(content: &[u8]) -> Result<String, lopdf::Error> {

    let operations = lopdf::content::Content::decode(content)?;
    let mut text = String::new();
    

    for operation in operations.operations {

        match operation.operator.as_str() {
            "Tj" | "'" | "\"" => {
                if let Some(Object::String(bytes, _)) = operation.operands.first() {

                    decode_pdf_text(bytes, &mut text);
                    text.push(' ');
                }
            },
            "TJ" => {
                if let Some(Object::Array(array)) = operation.operands.first() {
                    for item in array {
                        if let Object::String(bytes, _) = item {
                            decode_pdf_text(bytes, &mut text);
                        }

                    }
                    text.push(' ');
                }
            },
            _ => {} // Ignore other operators
        }
    }
    
    Ok(text)
}


fn decode_pdf_text(bytes: &[u8], output: &mut String) {

    if let Ok(s) = std::str::from_utf8(bytes) {
        output.push_str(s);
        return;
    }
    
    let mut decoder = String::new();
    let mut i = 0;
    while i < bytes.len() {
        let b = bytes[i];
        
        if i + 1 < bytes.len() && b == 0xFE && bytes[i+1] == 0xFF {
    
            i += 2;
            while i + 1 < bytes.len() {
                let high = bytes[i] as u16;
                let low = bytes[i+1] as u16;
                let utf16_char = (high << 8) | low;
                
    
                if let Some(c) = std::char::from_u32(utf16_char as u32) {
                    decoder.push(c);
                }
                i += 2;
            }
            break;
        }
        
    
        if b >= 32 && b <= 126 {
            // ASCII range
            decoder.push(b as char);
        } else {
            // Map some common non-ASCII characters
            match b {
                0x80..=0x9F => {
                    // Some common special characters in PDFDocEncoding
                    let c = match b {
                        0x91 => '\'',  // Left single quotation mark
                        0x92 => '\'',  // Right single quotation mark
                        0x93 => '"',  // Left double quotation mark
                        0x94 => '"',  // Right double quotation mark
                        0x96 => '–',  // En dash
                        0x97 => '—',  // Em dash
                        0x99 => '™',  // Trademark
                        0xA0 => ' ',  // Non-breaking space
                        _ => '�',     // Replacement character for unknown
                    };
                    decoder.push(c);
                },
                _ => {
                    
                    if let Some(c) = std::char::from_u32(b as u32) {
                        decoder.push(c);
                    } else {
                        decoder.push('�'); // Replacement character
                    }
                }
            }
        }
        i += 1;
    }
    
    if !decoder.is_empty() {
        output.push_str(&decoder);
    }
}