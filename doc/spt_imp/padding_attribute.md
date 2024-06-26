Der Wert `"on"` erzeugt Rahmen zur Auffüllung der Flächen um das Bild. Damit
ist das resultierende Bild immer so groß wie durch die Auflösung gefordert.
`padding=on` ist als Standardwert gesetzt, solange es nicht durch andere
Optionen ausgeschlossen ist.
Der Wert `"off"` erzeugt keinen Rahmen zur Auffüllung der Flächen um das
Bild. Damit ist das resultierende Bild unter Umständen kleiner als die
geforderte Auflösung.
Mit `"fit"` wird der größte mögliche Ausschnitt aus dem Originalbild, bzw.
aus dem durch `excerpt` gewählten Ausschnitt gesucht, bei dem das
Seitenverhältnis der geforderten Auflösung entspricht. Es wird kein Rahmen
erzeugt, sondern das Bild in einer Dimension gegebenenfalls gekürzt.
Um eine Abwärtskompatibilität zu gewährleisten, wird auch der Wert `"yes"`
(entspricht `"on"`) und `"no"` (entspricht `"off"`) unterstützt.