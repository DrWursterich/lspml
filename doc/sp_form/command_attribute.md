__Deprecated__. *Dieses Attribut ist veraltet und wird in zukünftigen
Versionen nicht mehr unterstüzt werden. Kommandos wurden in der Version 2.0
zugunsten einer flexibleren Lösung abgeschafft. Ein Kommando bestand aus einem
Template mit einem optionalen Handler. Für jede Template-Handler-Kombination
musste ein eigenes Kommando angelegt werden. Diese Verbindung wurde
aufgebrochen und durch zwei neue Attribute `template` und `handler` ersetzt. Um
einen Handler aufzurufen und anschließend ein Template auszuführen, ist nun
die Definition eines Kommandos nicht mehr nötig. Um einen Handler aufzurufen
und anschließend ein Template auszuführen, verwenden Sie die beiden Attribute
`handler` und `template`. Um einen Handler aufzurufen und anschließend eine
SPML-Seite auszuführen, verwenden Sie die Attribute `handler` und `uri`.*
Existierendes Command. Muss im GUI definiert worden sein.