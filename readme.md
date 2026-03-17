# NETCOM Client
This is a basic decentralized communication client that can be used to talk to other people through the internet, with verifiable identities and messages. The client here is an example, and there is a fairly detailed "protocol" section (specification) below, if anyone would want to recreate a compatible client. The specification section also includes other interesting technical information, so it might be worth a read. 

The client is pretty much fully-featured to the specification, though it's not *exactly* as declared. Mainly changing some configurations and numbers here and there, and doing other design choices that were left to client-implementation. Any built client isn't expected to adhere to the specification perfectly, however as long as your networking is decent enough, it should work. The rest are mainly rules so that messages actually flow instead of getting stuck somewhere. Though for the sake of the network, the more accurate the client, the better it is for everyone.

The final goal of NETCOM is to get a functional decentralized communication method that allows people to use it for whatever they really want, without much interference from others. But at the same time give a nice user experience, similar to other (centralized) messaging platforms, which includes not needing to self-host your own systems (which is common in other non-centralized platforms).

### Features
Right now it's a very barebones client supporting a barebones specification. It has:
- <b>Base messaging capability</b><br>
People can send other clients messages and other commands. Which is expected of a messaging client.
- <b>Message relaying between clients</b><br>
Message relaying is also a base feature of decentralized platforms. We need the messages to move, so we send them from computer to computer.
- <b>Tagging system for messages</b><br>
All messages can be tagged based on their contents. Expected to be used categorically, for example only displaying #rust or #coding etc etc.
- <b>Cryptographic user and message authentication</b><br>
All users must have a public key as authentication. This way you could send near-perfectly authenticated messages from anywhere to anyone, and no one can edit or repeat the message. Theoretically you could get your own tag and send messages to your server through it, though seems quite excessive.
- <b>Significant anonymity</b><br>
Anonymity is actually a nice side-effect from the decentralization setup. It is incredibly hard to determine your actual identity (in larger networks), only a general direction and an username/key (which cannot be used for full identification).

There are plans for future additions, though they will have to be updated with the entire spec. For version NCV2.1 there are some minor additions planned:
- Add proper backwards compatibility (kind of necessary)
- SYNC command for getting new peers
- FETCH command for getting older messages

There are also a significant amount of features that should be added to the example client, such as:
- Proper message filtering
- Blocking, ignoring and other social features
- Better and more featured UI
- Generic code reworks 

Other minor planned features can be seen in code marked as `TODO` or `FIXME`. Where TODOs mark an actual feature that will change the program, and FIXMEs as just generic code stuff.

## Usage
To use the example client you need to do some set up on the `data` folder. You need to create the files:

- `key` <br>
Here you need to type any 32 UTF-8 characters (specifically 32, no more no less). These will be converted to bytes and used as your private key, so don't share it around. **You should use an RNG to generate this**, though you can also smash your keyboard for a similar effect.
- `username` <br>
This is your username. You can type whatever UTF-8 here and it'll work, though keep it to at most 50 characters. You can change it freely, though keep in mind it's purely cosmetic. If you want a full identity change, change your key.
- `listener` <br>
Here will be the address used for creating the listener. More specifically, the interface address. Usually you want either `127.0.0.1:6500` for only accepting connections from the local machine, or `0.0.0.0:6500` for accepting connections from everywhere.
- `connection` <br>
This is your "initial host", or the IP address which will be used to get the first host for the network. Essentially, the IP you're connecting to.

After you've made these files, you can start the program and it should run as expected. Though there are also sections on both local and non-local setups.

The UI itself is pretty minimal (mostly meant for testing), and consists of two sections. There's the text and tag section at the bottom. You can insert some text into the upper section and tags at the lower section (use arrow keys to change section), then hit enter to send a message to the network.  Above that there's the messages section, which contains all the messages sent (note that your sent messages won't be visible unless they bounce back from another system, as to confirm that it did indeed reach somebody).

### Quick Start
A non-explained version of the setup guide

1. Clone the repo
2. Create files: <br>
&emsp;- `data/key` <br>
&emsp;Any 32 character (UTF-8) key, preferably cryptographically secure <br>
&emsp;- `data/username` <br>
&emsp;Any at most 50 character username <br>
&emsp;- `data/connection` <br>
&emsp;The IP Address you want to connect to, locally `127.0.0.1:6500` <br>
&emsp;- `data/listener` <br>
&emsp;The broadcast address, usually `0.0.0.0:6500` <br>
3. Run the program

### Local setup
The local setup is the recommended test environment if you're getting familiar with the project. Assuming you've done the previous setup <i>(Quick Start / Usage)</i>, you can simply start up two clients. (Normally, this should be enough, though for further information: )

The clients will start in <i>Server mode</i> and in <i>Connect-only mode</i> (in that order). These simply describe what capabilities they have. The server mode client will be able to accept connections, but hasn't been able to find a suitable client to connect to. On the other hand, the connect-only mode client isn't able to accept connections, but can send a connection request (for example, to the server client). This is because the port is taken by the other client, and is totally normal.

Generally, assuming you're using the default configuration, the clients should be able to find each other, and form a connection without any further setup. 

### Non-local setup
The non-local setup is a bit more challenging, but closer to an actual usage environment. You will first have to follow the base setup guide *(Quick Start / Usage)*, for the config, you want to pay special attention to `data/connection` and `data/listener`. In *connection*, you want to set the IP address of the computer you're connecting to. In *listener*, you most likely want to write `0.0.0.0:6500`, as that is the address for broadcasting yourself online\*.

If you start your first client, it should start itself in server-mode (see above on what it means), this is fine. You should start your second client, which should, assuming correct configuration, connect to the first one without issue.

\* Assuming you have set up port forwarding properly



# NETCOM Protocol
## Table of Contents
1.0 - Generics <br>
2.0 - Authentication and verification <br>
3.0 - Commands and parsing <br>
4.0 - Network <br>
&emsp; 4.1 - Network Joining <br>
&emsp; 4.2 - Command Relays <br>
5.0 - Lookups <br>
&emsp; 5.1 - Command list <br>
&emsp; 5.2 - Terms <br>

## 1.0 General
The NETCOM protocol is designed to work as a computer messaging protocol supporting multiple groups inside a single "network". It favors a more non-server specific architecture, where multiple computers "relay" messages across to each other. It is specifically designed to allow multiple groups of people to stay in the same "space" (network), yet stay separate by the use of basic filtering tags.

Even though the system prefers decentralized, non-server based networks, a common suggestion is to set up "boot servers" that work as the initial host for everyone connecting. The point is that everyone would connect to that server, which would then redirect you to another client. The point is, that even if that server was shut down or otherwise had issues, the network would stay online at least until everyone disconnects.

The protocol works by allowing computers to send "commands" to each other, with set arguments that are parsed into messages, authentication, and other things. See the command list at the end of the specification for a full list of commands that can be sent across the network. 

This document specifies the protocol version NCV2 (net-com-version-2)

See the "terms" list at the end of the specification for special terms used in the documentation, especially for repurposed terms. Most numbers/values given in the specification are configurable on a client-to-client basis, however all clients should support, at minimum, the given specification. 

## 2.0 Commands and Parsing
Commands are used to move around events that happen. Most notably commands are used to move messages, give join and leave events, as well as request joining the network. Basically, commands are the backbone of the protocol. Commands are sent as bytes using UTF-8 encoding, similarly to base HTTP. 

On the other hand, receiving, particularly parsing, commands is more client-specific. However, to ease parsing, commands are formatted in a certain way so that they can be split without issue. Specifically, they should be formatted using null bytes (`\0`) between any arguments, as well as one at the end of the command. Like this: `COMMAND \0 ARG1 \0 ARG2 \0 ARG3 ... \0`. It is quite important that the client ensures that no user content includes a null byte, these are to be simply removed, possibly replaced with a space, depending on client implementation. In case any null bytes do happen to get into your commands, the host should reject the command as malformed.

Any public keys sent should use the formatting `00-00-00-00-00...` (where 00 is an unsigned integer byte). This includes the ones used in signing. Relevantly, any timestamps are a 64-bit epoch in an integer.

## 3.0 Authentication
User authentication is done using Ed25519 based public-key signatures. After forming a connection to the network, the client will send a JOIN command. This command will include your username, public key, current epoch, a *salt* (see below), and then at the end, a signed string (an *"evidence"*) containing all the previous fields. This section would be formatted as `JOIN \0 USERNAME \0 PUB_KEY \0 EPOCH \0 SALT`. Clients should only accept JOINs that have an epoch of around 120 seconds off your own epoch (consider network delay between multiple computers). 

After authentication, clients can register your name and key, and keep it stored for further use (mainly, recognizing you later). Any proper identification should be done through the key, the username should be purely cosmetic. For clients, usually the best way to display usernames is the username and a short part of the public key (as someone can use the same username, but be authenticated through a different key).

The before-mentioned *evidence* will be used for any other commands sent inside the system. It will contain all the fields that are used with the command, prefixed with the command name, like this: `COMMAND \0 ARG1 \0 ARG2 \0 ARG3... \0`. (Note that the space between arguments and the null byte is purely for visual purposes, there should not be any spaces between them). 

The evidences will also include a small 8-character *salt*. This salt can be any 8 characters or numbers, you may use a any RNG to generate them, or even use a running counter between 0-99999999, it doesn't really matter. As long as each salt is different from the one sent before it. Salts should be scoped to each public key, meaning your salt shouldn't be used to discard other people's commands. Other clients may use this salt to identify whether they have seen your command before, and ignore any repeats. Clients should store all salts for around 2 minutes (or whatever acceptable epoch you have configured), since after that the message epoch goes invalid.

## 4.0 Networking
### 4.1 Network Joining
For new clients to join the network, they must first connect with TCP to some specified IP that is running NETCOM. The usual port for NETCOM is 6500, however any private networks may use whatever ports they desire (though to avoid confusion, it should stay the same across the network). After the initial TCP connection is formed, the host should send a READY command to the client to imply they're ready for their JOIN. The host may also send other commands instead, for example REDIR, which tells the client that this host does not want you to connect to it, and redirects you to someone else (who is connected to the same network). This can be used for example if you already have too many connections. Generally each host should have at most ~5 connections, and should redirect any new clients to other hosts, though larger purpose-built servers can theoretically host as many as they desire.

If for any reason someone, who the client thought was already JOINed, JOINs again, it is expected that they will update their username (if changed), and refresh them on any user lists the client might have. This is a method commonly used for changing usernames, though it may also imply that there is a desync or a block somewhere in the network.

### 4.2 Command Relays 
The network itself requires commands to travel around from computer to computer so that everyone can receive them. When hosts receive a command, they should first process it, and check for its validity. If the command is invalid (or a repeat), the host should simply ignore the command. If the command has no issues, the host should send it to everyone connected, except the client it came from.

## 5.0 Lookups
Things you might want to know or look up later for the protocol

### 5.1 Command list
Notice that the arguments used here are descriptive and don't use the needed formatting (see correct formatting in section §2.0). However they are in the correct order, so feel free to use this as the order guide. Standard fields used are explained below, any fields specific to one command are specified under them.

#### 5.1.1 Standard fields
- `username` <br>
Your username in a String, any characters fitting in UTF-8 are allowed, except obviously a null byte. Max length at around 50 characters
- `public-key` <br>
Your public key, formatted in `00-00-00...` where 00 is a byte. Keys should be 32 bytes total.
- `timestamp` <br>
A timestamp for the command, formatted as a single 64-bit number.
- `salt` <br>
An (up to) 8-character string, containing random characters or numbers. See §3.0 for specifics.
- `evidence` <br>
The signature for your command proving it has not been modified. See §3.0 for specifics

#### 5.1.2 Base command list
- `JOIN [username] [public-key] [timestamp] [salt] [evidence]` <br>
Sent when joining the network. Usually after you have formed the initial connection with the host.
- `LEAVE [username] [public-key] [timestamp] [salt] [evidence]` <br>
Sent when leaving the network. Uses the same forms as JOIN.
- `MSG [username] [public-key] [timestamp] [content] [tag(s)] [salt] [evidence]` <br>
Used for sending messages. Content field should include the contents of the message. Tags field should contain a set of tags used by clients for message sorting/filtering, each tag should be separated with a space. The maximum length of the content field should be around 10,000 characters, and the tag field 150 tags (of maximum 10,000 characters). Though that may be configurable if needed.

#### 5.1.3 Host to Client commands
- `REDIR [ip:port]` <br>
The host wants you to connect to this IP instead. Connection should close shortly after this command has been sent
- `READY` <br>
Implies that the host is ready for the JOIN command. Used when joining.
- `CLOSE` <br>
Implies that the host does not want you to connect. Used for private networks (IP-listed)

Clients should only need to expect HTC commands during joining. HTC commands after READY may be ignored.

### 5.2 Terms
There are some special, sometimes repurposed, terms that have been used when writing.

- *Client* : Some specific computer, usually the one we're focusing on.
- *Host* : The computer that a *client* has a connection to.
- *Documentation* / *Specification* : Refers to this specific document
- *Public/Private key* : Refers to digital signatures, public keys verify, private keys sign.
- *Evidence* : Refers to the ending of commands, that is a string that contains all previously mentioned fields in that exact order, signed using a private key.
- *Salt* : Refers to a random 8-character string used to give a command a "random ID"
- *HTC command* : Refers to a Host-to-Client command. See §5.1.3
- *Network* : Refers to a single interconnected system of computers running NETCOM
