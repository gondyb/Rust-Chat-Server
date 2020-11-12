# IRC-RS

GUYOT Gaston - GONDANGE Benjamin

## Rust IRC Server

Ce projet est une implémentation du protocole IRC comme décrit dans le memo [RFC 1459](https://tools.ietf.org/html/rfc1459).

## Lancement

`cargo run`

Le port est 3333.

## Connexion depuis un client IRC

Nous avons testé avec le client XChat irc pour linux, ainsi que le client CLI weechat-curses.

Pour se connecter au server, nous pouvons par exemple utiliser la commande

`/connect 127.0.0.1 3333`

## Implémentation

Les fonctions implémentées sont les suivantes :

* NICK s'authentifier
* JOIN pour rejoindre un salon
* PART pour quiter un salon
* PRIVMSG pour envoyer un message dans un salon
* QUIT pour quitter le serveur
* PING 

## Architecture

Des différents composants s'occupent des différentes parties de l'applocation :

* Registration s'occupe de l'enregistrement et du désenregistrement des clients
* Channels s'occupe de l'ajout et la suppression de clients aux channels
* Broadcast s'occupe d'envoyer des messages à une channel entière
* client_handler s'occupe de lire les messages envoyés par les clients et de les dispatcher.
* Postman s'occupe d'envoyer des messages uniques de manière asynchrone aux clients (peut provoquer des race conditions dans certaines situations)
* Protocol s'occupe de formatter les messages à envoyer