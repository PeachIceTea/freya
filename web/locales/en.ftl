# Generic errors
server-error--internal = The server encountered an unexpected error. Please try again later or contact the administrator.
server-connection-error = The API server couldn't be reached or returned an unexpected response. Please try again later or contact the administrator.

# General
app--title = Freya
date = { DATETIME($date, day: "numeric", month: "long", year: "numeric", hour: "numeric", minute: "numeric", timeZoneName: "short") }

# Navbar
navbar--login = Log in
navbar--logout = Log out
navbar--theme-dark = Dark
navbar--theme-light = Light
navbar--theme-system = System
navbar--theme-active = Active
navbar--edit-profile = Edit profile
navbar--admin = Administration
navbar--new-book = Add book
navbar--user-management = User management
navbar--library = Library
navbar--books = All books

# Authentication
server-authentication--already-logged-in = You are already logged in.
server-authentication--invalid-credentials = The provided username or password is incorrect.

login--title = Log in
login--input-username = Username
login--input-password = Password
login--button-login = Log in

# File system
server-fs--could-not-list-directory = The directory "{$value}" couldn't be read.
server-fs--ffprobe-failed = The file "{$value}" couldn't be read.

# Upload
server-upload--missing-data = The book couldn't be created because some data is missing.
server-upload--invalid-file-path = The file "{$value}" was not found on the server.

# Books
server-books--failed-to-get-cover-image = The server failed to get the cover image.

books--title = All Books
books--filter-library = Hide books already in your library

# Book details
book-details--title = {$title} by {$author}
book-details--title-placeholder = Book details
book-details--start-listening = Start listening
book-details--continue-listening = Continue listening
book-details--listen-again = Listen again
book-details--is-playing = Playing
book-details--add-to = Add to:
book-details--is-in = Is in:

# New book
new-book--title = Add a book to Freya
new-book--files-label = Audio files
new-book--select-file-button = {$multiSelect ->
    *[0] Select a file
    [1] Select files
}
new-book--files-selected = {$count ->
    *[0] No files selected
    [1] "{$fileName}" selected
    [other] {$count} files selected
}
new-book--modal--title = {$fileCategory ->
    *[image] Select a cover image
    [audio] Select audio files
}
new-book--modal-select-button = Select
new-book--modal-cancel-button = Cancel
new-book--modal-select-all-button = Select all files in this directory

new-book--title-label = Title
new-book--title-suggest = Suggest title: "{$title}"
new-book--author-label = Author
new-book--author-suggest = Suggest author: "{$author}"

new-book--cover-label = Cover image
new-book--cover-generic-explainer = To use it, keep this tab selected when you submit the form.

new-book--tab-cover-select = Select from server
new-book--cover-select-explainer = Select an image from the server to use as the cover image.

new-book--cover-url-placeholder = Image URL
new-book--cover-url-explainer = Enter the URL of the image you want to use as the cover.
new-book--tab-cover-url = Download from URL

new-book--tab-cover-upload = Upload
new-book--cover-upload-explainer = Upload an image from your computer to use as the cover image.

new-book--tab-cover-extracted = Use extracted image
new-book--cover-extracted-explainer = This image was extracted from the audio files you selected.

new-book--submit-button = Add book

# Player
player--volume = Volume
player--playback-speed = Playback speed

# User edit
user-edit--title = Edit user #{$id}
user-edit--not-llowed = You are not allowed to edit other users.
user-edit--name = Username
user-edit--password = Password
user-edit--password-confirm = Confirm password
user-edit--admin = Admin
user-edit--submit = Save

# User management
user-management--id = ID
user-management--title = User management
user-management--name = Username
user-management--admin = Admin
user-management--created = Created at
user-management--modified = Last modified
user-management--actions = Actions
user-management--show-profile = Show profile
user-management--edit = Edit
user-management--new-user = Add new user

# New user
new-user--title = Add a new user
