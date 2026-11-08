Der Wert `"on"` erzeugt Rahmen zur Auffüllung der Flächen um das Bild. Damit
ist das resultierende Bild immer so groß. wie durch die Auflösung gefordert.
Der Wert `"off"` erzeugt keinen Rahmen zur Auffüllung der Flächen um das
Bild. Damit ist das resultierende Bild unter Umständen kleiner als die
geforderte Auflösung.
Mit `"fit"` wird der größtmögliche Ausschnitt aus dem Originalbild bzw. aus
dem durch excerpt gewählten Ausschnitt gesucht, bei dem das Seitenverhältnis
der geforderten Auflösung entspricht. Es wird kein Rahmen erzeugt, sondern das
Bild in einer Dimension gegebenenfalls gekürzt. Ist das gewünschte Bild
größer als das Original wird das Bild wie bei `padding="on"` aufgefüllt.
Mit `"fit/no"` wird der größtmögliche Ausschnitt aus dem Originalbild bzw.
aus dem durch excerpt gewählten Ausschnitt gesucht, bei dem das
Seitenverhältnis der geforderten Auflösung entspricht. Es wird kein Rahmen
erzeugt, sondern das Bild in einer Dimension gegebenenfalls gekürzt. Ist das
gewünschte Bild größer als das Original wird das Bild nicht aufgefüllt.