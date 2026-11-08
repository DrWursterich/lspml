Bereich in dem die erzeugte Verbindung zum IES gespeichert werden soll.
- `windowSession` Verbindung wird nur für ein Browser-Fenster/Browser-Tab
verwendet (siehe `Window` Scope).
- `browserSession` Verbindung gilt für die komplette Browser-Instanz (siehe
`Session` Scope).
- `application` Verbindung gilt für das gesamte IES-Modul (Web-Applikation).
Bei Verwendung von `sp:login` in Live-Seiten ist dieser Scope zu empfehlen,
wenn immer der gleiche Nutzer verwendet wird (siehe `Application` Scope).
