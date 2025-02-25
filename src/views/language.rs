use enum_all_variants::AllVariants;
use serde::{Deserialize, Serialize};

use crate::generate_language_functions;

#[derive(Debug, AllVariants, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
pub enum Language {
    English,
    Deutsch,
}

generate_language_functions! {
    LanguageEnum: Language
    connections { 
        English: "Connections"
        Deutsch: "Verbindungen"
    }
    stats {
        English: "Stats"
        Deutsch: "Statistiken"
    }
    active_downlaods {
        English: "Active Downloads"
        Deutsch: "Aktive Downloads"
    }
    active_clients {
        English: "Active Clients"
        Deutsch: "Aktive Clients"
    }
    total_clients {
        English: "Total Clients"
        Deutsch: "Gesamte Clients"
    }
    total_downloads {
        English: "Total Downloads"
        Deutsch: "Anzahl Downloads"
    }
    current_upload {
        English: "Current Upload"
        Deutsch: "Aktueller Upload"
    }
    transmitted_data {
        English: "Transmitted Data"
        Deutsch: "Übertragene Daten"
    }
    upload_file {
        English: "Upload"
        Deutsch: "Hochladen"
    }
    no_file_selected {
        English: "No file selected!"
        Deutsch: "Keine Datei ausgewählt!"
    }
    drag_and_drop {
        English: "Drag and drop a file inside the window or click one of the buttons below to select files to share."
        Deutsch: "Ziehe eine Datei in das Fenster oder klicke auf einen der Knöpfe um Dateien zum Teilen auszuwählen."
    }
    select_files {
        English: "Select Files"
        Deutsch: "Dateien wählen"
    }
    select_folders {
        English: "Select Folder"
        Deutsch: "Ordner wählen"
    }
    select_folders_tooltip {
        English: "Share files from folders as individual files."
        Deutsch: "Teile Dateien aus Ordnern als einzelne Dateien."
    }
    zip_folder {
        English: "Zip Files"
        Deutsch: "Ordner zippen"
    }
    zip_folder_tooltip {
        English: "Share a compressed folder containing multiple files/folders retaining their structure."
        Deutsch: "Komprimiert die Dateien in einem Ordner in eine zip datei und teilt diese."
    }
    shared_files(n) {
        English: "Shared Files [{n}]"
        Deutsch: "Geteilte Dateien [{n}]"
    }
    cancel {
        English: "Cancel"
        Deutsch: "Abbrechen"
    }
    open {
        English: "Open"
        Deutsch: "Öffnen"
    }
    show {
        English: "Show"
        Deutsch: "Anzeigen"
    }
    delete {
        English: "Remove"
        Deutsch: "Entfernen"
    }
    delete_tooltip {
        English: "Cannot delete files while downloads are active."
        Deutsch: "Dateien können nicht gelöscht werden, solange Downloads aktiv sind."
    }
    remove_all {
        English: "Remove All"
        Deutsch: "Alle entfernen"
    }
    theme_tooltip {
        English: "You can change the theme of the application using the up and down arrow keys."
        Deutsch: "Du kannst das Farbschema der Anwendung mit den Pfeiltasten nach oben und unten ändern."
    }
    invalid_port(port) {
        English: "Invaid port number. Please enter a number between 0 and 65535. (Active Port: {port})"
        Deutsch: "Ungültige Portnummer. Bitte geben Sie eine Nummer zwischen 0 und 65535 ein. (Aktiver Port: {port})"
    }
    change_port(port) {
        English: "Press Enter to change the port. (Active Port: {port})"
        Deutsch: "Drücke die Eingabetaste(Enter), um den Port zu ändern. (Aktiver Port: {port})"
    }
    standard_port {
        English: "You can change the port the server is running on. If you want to serve the files on the internet, make sure to open the port in your router settings."
        Deutsch: "Hier kannst du den Port aud em der http server läuft änder. Wenn du die Dateien im Internet freigeben möchtest, stelle sicher, dass der Port in den Router-Einstellungen freigegeben ist."
    }
    locked_port(port) {
        English: "Cannot change the port while downloads are active. (Active Port: {port})"
        Deutsch: "Der Port kann nicht geändert werden, solange Downloads aktiv sind. (Aktiver Port: {port})"
    }
    language {
        English: "Language:"
        Deutsch: "Sprache:"
    }
    download {
        English: "Download"
        Deutsch: "Herunterladen"
    }
    copy_url {
        English: "Copy URL"
        Deutsch: "Kopieren"
    }
    open_in_browser {
        English: "Open"
        Deutsch: "Öffnen"
    }
    block_external_connections {
        English: "Block External Connections"
        Deutsch: "Externe Verbindungen blockieren"
    }
    connection_info {
        English: "Download URL"
    }
    block_external_connections_tooltip {
        English: "Block external connections to the server. Check this box if you want only devices on the local network to access the files."
        Deutsch: "Blockiere externe Verbindungen zum Server. Dadurch können nur Geräte im lokalen Netzwerk auf die Dateien zugreifen."
    }
    show_qr_code {
        English: "Show QR Code"
        Deutsch: "QR Code anzeigen"
    }
    show_connections {
        English: "Show Connections"
        Deutsch: "Verbindungen anzeigen"
    }
    downloading_tooltip(speed, progress, size) {
        English: "Downloading at up to {speed}/s\nProgress: ({progress}/{size})"
        Deutsch: "Lädt herunter mit bis zu {speed}/s.\n {progress} von {size} heruntergeladen."
    }
    last_download(duration, speed) {
        English: "Last download {duration} ago \nat up to {speed}/s "
        Deutsch: "Letzter Download vor {duration} \nmit bis zu {speed}/s "
    }
    connected {
        English: "Connected."
        Deutsch: "Verbunden."
    }
    last_seen(duration) {
        English: "Last seen {duration} ago."
        Deutsch: "Zuletzt gesehen vor {duration}."
    }
}

impl std::fmt::Display for Language {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}