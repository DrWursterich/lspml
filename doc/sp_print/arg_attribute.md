Mit diesem Attribut werden Werte für eine Formatierung im StringFormat
angegeben. Für dieses Attribut gilt der Sonderfall, dass mehrere Werte in
einzelnen `arg`-Attributen angegeben werden. Es ist also möglich mehrere
Attribute `arg` in diesem Tag anzugeben. Diese Formatierung wird durchgeführt,
wenn mindestens ein `arg`-Attribut angegeben wurde. Diese Formatierung wird
nach allen anderen Formatierungen (deciamlformat, numberformat), de- und
encodings und de- und encrypting durchgeführt. Die ermittelte Zeichenkette
wird zusammen mit den übergebenen Argumenten in den `arg`-Attributen nach den
Regeln des StringFormats formatiert. Zu beachten gilt, dass die `arg`-Argumente
eine Expression erwartet. Zahlen können direkt übergeben werden.
Zeichenketten müssen in ' gefasst werden
```spml
<sp:print text="a number: %d" arg="3"/>
<sp:print text="a word: %s" arg="'word'"/>
```