# Image backend (name pending)

## Features:

### Folder manipulation:
- ~~load folders and files~~ (multithreaded)
- ~~load files without loading folders~~
- ~~load folders without loading files~~
    - technically loads everything and filters
- ~~add single folder without loading files~~
- ~~clear selection of folders~~

### Image manipulation:
- Provide current image
- ~~Move image to selected folder~~
- ~~Delete image~~
- ~~Skip image~~
- maybe store to-be-deleted images in a temp folder and delete when memory is freed (when file can no longer be undone/redone)?

### Information displayed
- ~~Current directory~~
- Files remaining (if possible without it being slow)

### Flow control
- ~~Undo, redo stacks~~
- push control flow when moving
- push control flow when deleting
- perform undo
- how to undo a delete in rust? not possible?
