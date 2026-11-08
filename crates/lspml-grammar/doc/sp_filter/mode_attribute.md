Auswahl des Filter-Mechanismus.
__simple (Wildcard-Filter)__
Der Filter kann die Wildcards `*` für beliebige Zeichen und `?` für ein
beliebiges Zeichen enthalten. So würde eine wie folgt gefilterte Liste nur
Elemente enthalten, die mit a beginnen.
```regex
a*
```
__regex (Reguläre Ausdrücke)__
Für komplexe Filter stehen Reguläre Ausdrücke (POSIX) zur Verfügung. So
würde im regex-Filtermode eine mit
```regex
[a-dA-D].*
```
gefilterte Liste nur Elemente enthalten, die mit dem Buchstaben A, a, B, b, C,
c, d oder D beginnen.