FROM node:20.15.0

WORKDIR /frontend

COPY package*.json ./

RUN npm install

COPY . .

ENV PORT=8080

EXPOSE 8080

CMD [ "npm","start" ]
