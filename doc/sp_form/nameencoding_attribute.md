Die innerhalb von sp:form liegenden Input-Tags (`sp:text`, `spt:text`,
`sp:checkbox`, ...) erhalten vom IES generierte Feldnamen, die unter Umständen
(wenn sie z.B. innerhalb von `sp:iterator` liegen) Sonderzeichen wie eckige
Klammern (`[`, `]`) enthalten können. Beim Aufbau von Live-Seiten, die in PHP
eingebettet sind, wird das Formular an PHP-Seiten gesendet. Da
Request-Parameternamen mit Sonderzeichen von PHP nicht richtig ausgewertet
werden, ist es mit diesem Attribut möglich, die Formularfeldnamen zu encoden,
damit keine Sonderzeichen mehr enthalten sind. Vom IES unterstüzte Encodings
für Feldnamen sind:
- `escff` *(default)* Wandelt nur die Zeichen des Feldnamens um, die zu Fehlern
führen können z.B. Eckige Klammern (`[]`). Beispiel: Aus
`sp_iterator[1].sp_body` wird `escff:sp_iterator:5b:1:5d::2e:sp_body.` Dieses
Encoding ist kein Standard-Encoding, sondern eine proprietäre Entwicklung von
Sitepark.
- `hex` Wandelt jedes Zeichen des Feldnamens in den entsprechenden Hex-Wert um.
Beispiel: Aus `sp_body` wird "hex:73705f626f6479"