__Deprecated__. *Dieses Attribut wird nicht mehr unterstützt.*
Diese Option schneidet einen Ausschnitt eines größeren Bildes aus. Die ersten
beiden Zahlen geben die linke obere Ecke des Ausschnittes an, die letzteren
beiden die untere rechte Ecke. Mögliche Werte sind x0,y0,x1,y1 z.B.
100,100,300,200. Dieser Ausschnitt wird entsprechend der Optionen `height` und
`width` noch verkleinert oder vergrößert. Hierbei wird gegebenenfalls ein
Rand erzeugt, sprich die Option `padding=yes` ist automatisch gesetzt, falls
nicht `padding=fit` gesetzt ist.
Alle 4 Zahlen können auch negativ sein. In diesem Fall wird der Wert als
Differenz zum hinteren oder unteren Rand des Bildes berechnet. Also bedeutet
-10% dasselbe wie 90% und -100 bei einem 300 Pixel breiten (oder hohen) Bild
dasselbe wie 200. Ist `x0 > x1`, wird das Bild an der `x`-Achse gespiegelt.
Ist `y0 > y1`, wird das Bild an der `y`-Achse gespiegelt. Mit Angabe der Werte
`x0,y0` z.B. 100,50 wird der Ausschnitt in der exakten Größe der mittels
`height` und `width` geforderten Auflösung gewählt. Es ist dann keine
Verkleinerung oder Vergrößerung mehr notwendig und man erhält einen 1:1
Ausschnitt des Orignals. Hierbei ist immer `padding=no` gesetzt.
Mit den Variablen `north`, `west`, `east` oder `south` wird ein in der
jeweiligen Himmelsrichtung gelegener Ausschnitt in der mittels `height` und
`width` geforderten Auflösung gewählt. Also wird mit `excerpt=south` ein
Ausschnitt auf der Mitte der Bildbreite ganz unten gewählt, mit `excerpt=east`
dagegen ein Ausschnitt aus der Mitte der Bildhöhe ganz rechts. Es ist dann
keine Verkleinerung oder Vergrößerung mehr notwendig und man erhält einen
1:1 Ausschnitt des Orignals. Hierbei ist immer `padding=no` gesetzt.
Mit northwest, northeast, southwest oder southeast wird ein in der jeweiligen
Himmelsrichtung gelegener Ausschnitt in der mittels `height` und `width`
geforderten Auflösung gewählt. Also wird mit `excerpt=southeast` die
äußerste untere, rechte Ecke des Originalbildes gewählt, mit
`excerpt=northwest` dagegen die obere, linke Ecke. Es ist dann keine
Verkleinerung oder Vergrößerung mehr notwendig und man erhält einen 1:1
Ausschnitt des Orignals. Hierbei is immer `padding=no` gesetzt.