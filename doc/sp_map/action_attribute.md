Aktion, die ausgeführt werden soll. Die folgenden Aktionen sind möglich:
`put`, `remove`, `new` und `clear`.
- `put` Träg ein neues Schlüssel-Werte-Paar in die Map ein. Existiert schon
ein Eintrag mit dem angegebenen Schlüssel, wird der alter Wert überschrieben.
- `putNotEmpty` Träg ein neues Schlüssel-Werte-Paar in die Map ein, wenn der
Wert nicht null oder ein Leerstring ist. Existiert schon ein Eintrag mit dem
angegebenen Schlüssel, wird der alter Wert überschrieben.
- `putAll` Bei dieser Aktion muss eine weitere Map übergeben werden. Alle
Einträge werden in die Map übernommen.
- `merge` Bei dieser Aktion muss eine weitere Map übergeben werden. Alle
Einträge werden in die Map übernommen. Enthält die Map aber weitere
Map-Strukturen, werden diese zusammengeführt. Bei der Merge-Aktion werden
immer Kopien der Daten in die Map übernommen. Bei putAll sind es immer
Referenzen. Wie bei putAll werden alle Eintäge in die Map übernommen.
- `remove` Löscht das Schlüssel-Werte-Paar mit dem in `key` angegebenen
Schlüssel aus der Map.
- `new` Erzeugt eine neue Map
- `clear` Löscht den Inhalt der Map
