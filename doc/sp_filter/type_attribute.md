Der Typ für die from und to Attribute: `number`, `text`, `date`.
- `number` Das Element oder Attribut wird als Zahl interpretiert. Es wird nicht
herausgefiltert wenn es innerhalb des Zahlenbereiches liegt, der mit `from` und
`to` definiert wurde.
- `text` Das Element oder Attribut wird als Text interpretiert. Es wird nicht
herausgefiltert wenn der Text mit den Zeichen beginnt, die in dem mit `from`
und `to` definierten Bereich liegen.
- `date` Das Element oder Attribut wird als Datum interpretiert. Es wird nicht
herausgefiltert wenn es innerhalb des Datumbereiches liegt, der mit `from` und
`to` definiert wurde.