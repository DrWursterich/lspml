Innerhalb einer (`Session`) können für jedes Browserfenster weitere
`Windowsessions` existieren. Dies ist sinnvoll, wenn die Session über ein
Cookie gehalten wird und dennoch unterschiedliche Sessions in einem Browser
benötigt werden. Existiert so eine Windowsession wird die `ID` dieser Session
mit an die URL gehangen. Um dies zu verhindern, muss dieses Attribut auf
`false` gesetzt werden.