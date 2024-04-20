SPML-Seiten sind immer Teil einer Webapplikation. Jede Webapplikation besitzt
eine Context-Pfad mit dem die URL beginnt (Es existert auch ein
ROOT-Context-Pfad (`/`)). Soll die URL einer Seite herausgeschrieben werden,
die in einer anderen Webapplikation liegt, so wird mit diesem Attribut der
Context-Pfad angegeben. Context-Pfade von Webapplikationen können sich
ändern. Damit auch bei solchen Änderungen die URL richtig generiert wird,
sollte in den meisten Fällen eher das Attribut `module` verwendet werden.