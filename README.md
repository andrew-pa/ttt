# ttt

A visual Vim-like editor for trees of text blobs, like outlines or nested TODOs.

![image](https://user-images.githubusercontent.com/6148347/206799635-01996d58-2c5d-489d-a29b-c3f5dbedc0c2.png)

## Usage

`ttt` has a Vim-like modal interface, but adds a new top-level mode for manipulating the actual tree called tree mode.
Editing the text of a single mode is done in edit mode and insert mode that work like Vim's normal and insert mode, except scoped to a single node of text.
Not all Vim motions or commands are supported yet, but the basics are there.

Rather than using registers, cut/copy/paste operate using a "snip stack", one for entire tree nodes and a separate one for text.

When `ttt` loses focus it will automatically sync the current tree if it has a location.

### Tree mode commands

| Key | Command                                     |
|-----|---------------------------------------------|
|  j  | Move to next child (visually "down" tree)   |
|  k  | Move to previous child (visually "up" tree) |
|  l  | Move to the first child of the current node |
|  h  | Move to the parent of the current node      |
|  i  | Start inserting in current node             |
|  e  | Start editing current node                  |
|  o  | Insert node in parent after current node    |
|  O  | Insert node as child of current node        |
|  x  | Cut a node onto the snip stack              |
|  y  | Copy a node onto the snip stack             |
|  p  | Paste the top node of the snip stack        |
|  P  | Pop a node off the snip snack and insert it |
|alt+p| Paste the top node of the snip stack as a child |
|alt+P| Pop a node off the snip snack and insert it as a child |
|  f  | toggle current node being folded (collapsed)|
|  r  | set the current node as the current displayed root |
|  :  | enter command mode                          |

### Command mode commands

- `e <url>`: start editing a tree stored at `<url>`
- `s (<url>)`: sync the current tree with the stored version, optionally setting the location url
- `q`: quit

## Storage

Right now `ttt` supports storing and loading trees locally as text files in the [RON](https://github.com/ron-rs/ron) format.

Storage locations can be specified with URLs or using local paths starting with `.` or `~`. Right now the `file://` protocol is also supported for absolute paths.

## Building

You should just be able to run `cargo build`. Your platform must be able to support Skia on OpenGL. Metadata is provided to run `cargo bundle` as well to create an application bundle.
