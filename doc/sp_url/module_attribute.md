SPML-Seiten sind immer Teil einer Webapplikation. Jede Webapplikation besitzt
einen Context-Pfad mit dem die URL beginnt (Es existert auch ein
ROOT-Context-Pfad (`/`)). Soll die URL einer Seite herausgeschrieben werden,
die in einer anderen Webapplikation liegt, so wird mit diesem Attribut die ID
dieser Webapplikation angegeben. Somit wird die URL auch dann richtig erzeugt,
wenn sich der Context der Ziel-Webapplikation ändert.