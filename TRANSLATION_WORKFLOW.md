# Cookbook GTK i18n/translation workflow

# 1. Extract translatable strings from Rust source files to cookbook.pot
xgettext --language=Rust --keyword=tr --output=cookbook.pot ../cookbook-gtk/src/*.rs

# 2. Create a new translation file for a language (e.g., German)
msginit --input=cookbook.pot --locale=de --output-file=de.po

# 3. Edit de.po and translate the strings
# (You can use a text editor or a tool like Poedit)

# 4. Compile the .po file to a .mo file
msgfmt de.po -o cookbook.mo

# 5. Install the .mo file in the correct directory structure for testing
mkdir -p ./locale/de/LC_MESSAGES
mv cookbook.mo ./locale/de/LC_MESSAGES/

# 6. Run the app with LOCPATH set to use your local translations
# (This tells gettextrs to look in ./locale for .mo files)
LOCPATH=$(pwd)/locale LANGUAGE=de cargo run -p cookbook-gtk

# Repeat steps 2-5 for other languages (fr, es, etc.)

# Notes:
# - The domain name is 'cookbook' (so the .mo file must be named cookbook.mo)
# - You can update translations by editing the .po file and recompiling.
# - For system-wide install, .mo files go in /usr/share/locale/<lang>/LC_MESSAGES/cookbook.mo
