fn main() {
    println!("Hello, world!");

    let pdf_path = "paper.pdf";
    let password = "";

    use poppler::PopplerDocument;

    let pdf = PopplerDocument::new_from_file(&pdf_path,Some(password)).map_err(|e|{
        eprintln!("error in reading file {e}");
    }).unwrap();
    let count = pdf.get_n_pages();

    println!("{count}");

    let mut result = String::new();

    for i in 0..count{
        let page = pdf.get_page(i).unwrap();
        if let Some(content) = page.get_text(){
            println!("{content}");
            result.push_str(content);
            result.push_str("\n");
        }
    }
}
