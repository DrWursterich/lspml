Rate mit der das Bild komprimiert wird. Die Werte liegen zwischen 1 und 100. Wobei 1 einer niedrige Qualität bzw. hohen Kompression und 100 einer hohen Qualität bzw. niedrige Kompression entspricht. Der angegeben Wert hat je nach Bildformat (gif, png, jpg) unterschiedlich interpretiert (siehe [hier](https://www.imagemagick.org/script/command-line-options.php#quality%7Chier)). Um für die unterschiedlichen Bildformate differenzierte Qualitätsstufen angeben zu können werden diese Kommasepariert Wertepaare mit Doppelpunkt-Trenner angegeben.
__Einfache Angabe__
```
60
```
__Spezifische Angabe__
```
gif:70,png:50,jpg:62
```