SPML-Seiten sind immer Teil einer Webapplikation. Die mit dem Attribut `uri`
angegebene SPML-Seite bezieht sich immer auf die aktuelle Webapplikation. Soll
eine Seite einer anderen Webapplikation eingebunden werden, so wird mit diesem
Attribut der Context der Webapplikation angegeben. Da sich der Context einer
Webapplikation ändern kann, ist in den meisten Fällen die Verwendung des
Attributes `module` zu empfehlen, da hier die ID der Webapplikation angegeben
wird.