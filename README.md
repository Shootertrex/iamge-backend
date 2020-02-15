# Image backend (name pending)

## Features:
- Moving and deleting files will require manually calling the method to move forward the pointer
    - This reduces the complexity of each method to only return 1 type of error

### Folder manipulation:
- ~~load folders and files~~ (multithreaded)
- ~~load files without loading folders~~
- ~~load folders without loading files~~
    - technically loads everything and filters
- ~~add single folder without loading files~~
- ~~clear selection of folders~~

### Image manipulation:
- ~~Provide current image~~
- ~~Move image to selected folder~~
- ~~Delete image~~
- ~~Skip image~~
- maybe store to-be-deleted images in a temp folder and delete when memory is freed (when file can no longer be undone/redone)?

### Information displayed
- ~~Current directory~~
- Files remaining (if possible without it being slow)

### Flow control
- ~~Undo, redo stacks~~
- ~~push control flow when moving~~
- ~~push control flow when deleting~~
- ~~perform undo~~
- ~~perform redo~~
- how to undo a delete in rust? not possible?

### Other
- index out of bounds when getting current file when no files loaded (usually on program start up)
- index out of bounds on moving files when no folders (may not bug with this. still investigate)
- filter out duplicate folders when adding/loading
- write documentation
- can't use ~/ in folder names
- maybe don't want to wipe out folders when loading external
- allow multiple source folders for files
