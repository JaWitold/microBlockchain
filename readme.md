# Blockchain Microservices Application
This is a blockchain application based on the microservices architecture. The project is composed of three directories: [backend](#backend), [frontend](#frontend), and [scripts](#scripts).

## About the Project
This is a group project made by students studying computer science in their second degree program, for their cryptography classes. The purpose of the project is to apply the concepts and principles learned in the course to create a blockchain application based on microservices architecture.

## Structure
### Backend
The backend directory contains a Rust application that serves as a blockchain miner. Its purpose is to connect to other backends in peer-to-peer (P2P) mode. The backend communicates with other miners to validate transactions and reach a consensus on the current state of the blockchain.

### Frontend
The frontend directory contains a Next.js 13 application with TypeScript. It serves as a REST API that lets users put data into the blockchain, read blocks, and view the entire chain. Users can interact with the blockchain through HTTP requests to the REST API.

### Scripts
The scripts directory contains Python scripts used to generate Docker Compose files with multiple stacks of this project to simulate the behavior of the blockchain locally. Each stack is composed of the frontend, backend, Redis, and RabbitMQ, with each stack having its own network. The use of multiple stacks allows for simulation of different nodes in the network.

## Running the Application
Before running the application, please ensure that you have the following tools installed on your machine:

- Docker
- Docker Compose
- Rust
- Python 3
    - yaml module (`pip3 install pyyaml`)
- Node.js v18 or higher
- Makefile (optional, commands specified in the Makefile can be run manually)

*Please note that on Windows, it is recommended to run this application using WSL2 to ensure that all dependencies are properly configured.*

To successfully build and run the containers, ensure that Docker is running on your system. Open your terminal and navigate to the root directory of the project. From there, execute the following command using the provided Makefile:

``` bash
make n=3
```
This command will generate the Docker Compose file using the Python script in the scripts directory, build the containers, and start the application.

Please note that while it is possible to run the backend and frontend locally, it may require installing Redis and RabbitMQ locally, which is not recommended. It is recommended to use the Docker Compose setup to ensure that all dependencies are properly configured.

If you need to stop the application, you can run:

``` bash
make down
```

And if you need to start the application again, you can run:

``` bash
make up
```

If you need to rebuild the containers, you can run:

``` bash 
make build n=3
```
To stop the application and remove any orphaned containers, you can run:

``` bash
make kill
```
And to restart the application, you can run:

``` bash
make restart
```

Please feel free to reach out if you encounter any issues or have any questions about running the application.

## Contributors
This project was created by a group of students, including:

- Krzysztof Tałałaj (254653@student.pwr.edu.pl),
- Denys Korniienko (256200@student.pwr.edu.pl), 
- Witold Karaś (254622@student.pwr.edu.pl)

## Acknowledgements

We would like to thank our professor for providing us with the opportunity to work on this project and for guiding us throughout the development process.

## License
This project is licensed under the MIT License. See the LICENSE file for details.
