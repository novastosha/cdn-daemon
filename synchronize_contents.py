import json
import pathlib

# Make the directory listing into a seperate function so we could use it in the main function and in a recursive manner
def generate_directory_contents(directory, generate_index_html=False):
    contents = []
    for item in directory.iterdir():
        if item.is_file():
            if item.name in  ['contents.json', 'index.html', '.gitignore']:
                continue

            contents.append(item.name)
        elif item.is_dir() and item.name not in ['.git', 'logs']:
            contents.append({
                'name': item.name,
                'type': 'directory',
                'path': str(item.relative_to(directory))
            })

            generate_directory_contents(item, True)  # Recursively generate contents for subdirectories
    
    if generate_index_html:
        index_html_path = directory / 'index.html'
        if not index_html_path.exists():
            with open("../assets/c/directory_template.html", 'r') as template_file:
                template_content = template_file.read()
            with open(index_html_path, 'w') as index_file:
                index_file.write(template_content)


    contents.sort(key=lambda x: x if isinstance(x, str) else x['name'].lower())

    with open(directory / 'contents.json', 'w') as f:
        json.dump(contents, f, indent=4)

def main():
    root = pathlib.Path(".")
    generate_directory_contents(root)
if __name__ == "__main__":
    main()