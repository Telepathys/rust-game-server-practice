FROM node:16-alpine
WORKDIR /usr/src/app
COPY package.json ./
RUN yarn 
COPY . .
RUN yarn install
CMD [ "yarn", "dev" ]