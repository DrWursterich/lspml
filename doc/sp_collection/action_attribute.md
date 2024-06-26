Aktion, die ausgeführt werden soll. Es existieren die Aktionen `add`,
`addAll`, `remove`, `clear`, `new`, `replace`, `removeFirst`, `removeLast` und
`unique`.
- `add` Fügt ein Element am Ende der Liste ein. Ist ein `index` angegeben, so
wird das Element an dieser Position eingefügt. Das ursprüngliche Elemente und
alle nachfolgenden Elemente werden eine Position weiter geschoben.
- `addNotEmpty` Fügt ein Element am Ende der Liste ein, wenn der Wert nicht
`null` oder ein Leerstring ist. Ist ein `index` angegeben, so wird das Element
an dieser Position eingefügt. Das ursprüngliche Elemente und alle
nachfolgenden Elemente werden eine Position weiter geschoben.
- `addAll` Mit dieser Aktion können mehrere Elemente der Liste hinzugefügt
werden. Dazu muss `object` vom Typ `Collection` sein.
- `remove` Löscht ein Element aus der Liste. Ist `index` angegeben, wird das
Element an der Index-Position gelöscht und alle nachfolgenden Elemente
rutschen eine Position nach oben. Ist `object` bzw. `value` angegeben, wird das
Element in der Liste gesucht und gelöscht.
- `clear` Löscht alle Elemente aus der Liste.
- `new` Erzeugt eine neue leere Liste.
- `replace` Ersetzt ein Element der Liste. `index` gibt hierbei die Position
des Elements an, das durch `object` bzw. `value` ersetzt werden soll.
- `removeFirst` Löscht das erste Element der Liste.
- `removeLast` Löscht das letzte Element der Liste.
- `unique` Entfernt alle mehrfach vorkommenden Elemente aus der Liste.
- `insert` Fügt ein Element ein und verschiebt alle nachfolgenden Elemente um
eine Position. Wenn in eine Position eingefügt wird, die noch nicht belegt
ist, wird das delta mit `null` aufgefüllt.