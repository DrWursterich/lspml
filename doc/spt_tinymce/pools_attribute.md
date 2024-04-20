Kommaseparierte Liste von `Anchor` von Artikelpools oder von `ID`s von
Artikelpools; die Elemente der Pools werden dem Redakteur in einem Linkdialog
innerhalb des Editors zur Auswahl angeboten. Voraussetzung, dass der interne
Linkdialog überhaupt angezeigt wird, ist die Konfiguration des Editors mit
`iesLink` über `theme_advanced_buttons` in der
[TinyMCE:Configuration](http://wiki.moxiecode.com/index.php/TinyMCE:Configuratio
n). Ausgabeseitig muss man darauf achten, dass ein interner Link vor der
Ausgabe mit `spt:id2url` umgewandelt wird.