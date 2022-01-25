# `to-trash` 🚮

`to-trash` (`tt` for short) is a fast, small, and hopefully FreeDesktop-compliant file trasher for Linux.

## Compliance

`tt` aims to have compliance with the [FreeDesktop.org Trash specification](https://specifications.freedesktop.org/trash-spec/trashspec-1.0.html).

Checked items below are what `tt` considers to be implemented, unchecked is anything that has no or partial implementation.

Some of those are my interpretation of the spec. and not necessarily verbatim to the specification text.

* [x] Considers that the "home trash" is located at `$XDG_DATA_HOME/Trash`.
    * If `XDG_DATA_HOME` is not defined, falls back to `~/.local/share/Trash`.
* [x] Files that the user trashes from the same mount point as home are stored in the home trash.
* [x] Trashed files are sent to `$trash/files`. 
* [x] An *info file* is created for every file being trashed.
    * [x] Contains a `Path` key with the absolute pathname of the original location of the file/directory
    * [x] Contains a `DeletionDate` key with the date and time when the file/directory was trashed in the `YYYY-MM-DDThh:mm:ss` format and in the user's local timezone.
* [x] Create or update the `$trash/directorysizes` file, which is a cache of the sizes of the directories that were trashed into this trash directory.
    * [x] Each entry contains the name and size of the trashed directory, as well as the modification time of the corresponding trashinfo file
    * [x] The size is calculated as the disk space used by the directory and its contents.
    * [x] The directory name in the directorysizes must be percent-encoded.
    * [x] To update this file, a temporary file followed by an atomic rename() operation must be used in order to avoid corruption due to two implementations writing to the file at the same time.
    * [ ] Note: the implementation currently calculates the total size of the directory in bytes. I'm not sure if this is what the standard meant.
* [x] If a `$topdir/.Trash` does not exist or has not passed the checks:
    * [x] If a `$topdir/.Trash-$uid` directory does not exist, the implementation must immediately create it, without any warnings or delays for the user.

Feel free to open an issue if you feel like `tt` is lacking any important features.

