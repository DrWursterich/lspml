Wird der Collection-Tag in Verbindung mit Suchabfragen verwendet (durch `query`
oder `object`), ist ein Publikationsbereich erforderlich, mit der die
Suchabfrage ausgeführt werden soll. Mit diesem Attribut können ein oder
mehrere Publikationsbereiche angegeben werden (durch Kommata getrennt).
Entweder werden die Publikationsbereiche durch ihren Anchor angegeben, oder
folgende Schlüsselwörter verwendet:
- `current` Der aktuelle Publikationsbereich. Dieser steht im `out`- und
`preview`-Modus als default-Wert zur Verfügung.
- `ignore` Ignoriert die Publikationsbereiche und liefert die Treffer
unabhängig davon, ob sie publiziert sind oder nicht.
- `all` Liefert die Treffer, wenn sie in irgendeinem der dem Mandanten
zugewiesenen Publikationsbereiche publiziert sind.
- `auto` Entspricht im `out`- und `preview`-Modus dem Schlüsselwort `current`
und im `in`-Modus `ignore`.