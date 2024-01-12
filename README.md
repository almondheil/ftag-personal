# ftag

A utility to tag files, list tags, and search for files with certain tags.

Meant to help categorize a large body of files
(such as images)
where each image fits into one or more categories
(say a photo is of a landscape, but also from a particular trip).

This is my attempt at making something that can organize that type of data.

# Installation

This is my first time really using Rust, so I don't know how to get it installed as a system utility yet.
If I learn that, this will get updated.

Until then, cloning this repo and moving or symlinking the executable somewhere gets the job done (pretty much).
Here's an example of what you might do.

First, clone the repo and cd into it.

```
git clone https://github.com/almondheil/ftag.git
cd ftag
```

Then, build the release version of the utility.

```
cargo build --release
```

Finally, move the executable created to somewhere you actually want it on your system
that's in `$PATH`.

```
mv target/release/ftag ~/.local/bin/
```

# Example usage

## Initialize the database

To do anything, you must initialize the database.
This should be done in the root directory of where you store the files,
as you need to be in the same directory as the database to perform any actions.

This was done to simplify operation of the tool and make it clear that 
it's not meant to be able to index and tag an entire system.

```
$ ftag init
Initialized database.
```

## Add tags to a file

You can add tags to a new file like so:

```
$ ftag add example.jpg landscape red-rocks
landscape
red-rocks
```

Or add more tags to a file that you already tagged:

```
$ ftag add example.jpg pretty
landscape
pretty
red-rocks
```

If you try to add duplicate tags, they'll just be ignored:

```
$ ftag add example.jpg red-rocks pretty
landscape
pretty
red-rocks
```

## Remove tags from a file

You can remove tags from a file:

```
$ ftag rm example.jpg pretty
landscape
red-rocks
```

If you try to remove a tag that doesn't exist, it won't do anything:

```
$ ftag rm example.jpg evil
landscape
red-rocks
```

## Swap a tag name

Meant as a convenience if you ever misspell a tag when typing it, or need to rename categories. All it does is remove the original tag and add a new one.

```
$ ftag rename example.jpg landscape landscape-photo
landscape-photo
red-rocks
```

If you try to swap from an old name that doesn't exist, it won't do anything.

## List the tags of a file

You can list in normal alphabetic order:

```
$ ftag list example.jpg
landscape-photo
red-rocks
```

Or in reverse if that's useful to you:

```
$ ftag list example.jpg -r
red-rocks
landscape-photo
```

## List all the tags in the database

Simply don't specify a filename to list.
Let's pretend I added tags to some other files to make a better case for the usefulness of this option:

```
$ ftag list
landscape-photo
portrait-photo
red-rocks
starred
yosemite
```

You can also include the number of times each tag is used:

```
$ ftag list -c
(2) landscape-photo
(2) portrait-photo
(3) red-rocks
(1) starred
(1) yosemite
```

You can also sort by usage:

```
ftag list -cs
(3) red-rocks
(2) landscape-photo
(2) portrait-photo
(1) starred
(1) yosemite
```

And of course, reverse this listing as well:

```
$ ftag list -csr
(1) yosemite
(1) starred
(2) portrait-photo
(2) landscape-photo
(3) red-rocks
```
(this functionality actually helped me catch and correct my own misspelling of the `portait-photo` tag while preparing this example.)

## Find files with certain tags

You can find files that have certain tags:

```
$ ftag find red-rocks
example.jpg
example2.jpg
example4.jpg
```

You can also exclude other tags at the same time.
The syntax for this is less comfortable than I wanted,
but it's what I could figure out how to make work. 
Suppose that `example2.jpg` is a portrait photo.

```
$ ftag find red-rocks -- portrait-photo
example.jpg
example4.jpg
```

## Deleting the database

I don't provide a command to do this (yet, at least), 
but the entire database is just stored in the file `.ftag.db`. 
Removing it will delete all tags.
