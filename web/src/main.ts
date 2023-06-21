import './style.css';
import {addedDiff, deletedDiff, diff, updatedDiff} from 'deep-object-diff';

const paper = document.createElement('p');
paper.textContent = 'Hello World!';
paper.style.color = '#fff';
paper.style.fontSize = '12px';
paper.style.position = 'fixed';
paper.style.left = '0px';
paper.style.top = '0px';
paper.style.padding = '10px';
paper.style.fontFamily = 'Consolas, monospace';
paper.style.whiteSpace = 'pre';
document.body.appendChild(paper);

const canvas = document.createElement('canvas');
canvas.width = 800;
canvas.height = 600;

document.body.appendChild(canvas);

class Conversation<T extends unknown> {
  constructor(public kind: string, public data?: T) {}

  toString() {
    return JSON.stringify(this);
  }
}

class Action<T> {
  callbacks: ((data: T) => void)[] = [];

  invoke(data: T) {
    for (let i = 0; i < this.callbacks.length; i++) {
      this.callbacks[i].call(null, data);
    }
  }

  subscribe(callback: (data: T) => void) {
    this.callbacks.push(callback);
    return () => this.unsubscribe(callback);
  }

  unsubscribe(callback: (data: T) => void) {
    this.callbacks = this.callbacks.filter(cb => cb !== callback);
  }
}

class Provider {
  socket: WebSocket;
  actions: { [key: string]: Action<any> } = {};

  constructor() {}

  async connect() {
    return new Promise<void>((resolve, reject) => {
      this.socket = new WebSocket('ws://localhost:1111/');
      this.socket.onopen = () => {
        resolve();
      }

      this.socket.onerror = (err) => {
        reject(err);
      };

      this.socket.onmessage = (msg) => {
        const { kind, data: buf } = JSON.parse(msg.data);
        const data = JSON.parse(buf);

        if (this.actions[kind]) {
          this.actions[kind].invoke(data);
        }
      };
    });
  }

  send<T extends unknown>(kind: string, data?: T) {
    this.socket.send(new Conversation(kind, data).toString());
  }

  on<T extends unknown>(kind: string, callback: (data: T) => void) {
    if (!this.actions[kind]) {
      this.actions[kind] = new Action<T>();
    }

    return this.actions[kind].subscribe(callback);
  }
}


class Vector2f {
  constructor(public x: number, public y: number) {}

  clone() {
    return new Vector2f(this.x, this.y);
  }
}

class Entity {
  id: string;
  game: Game;
  kind: string;

  start() {

  }

  destroy() {

  }

  update(delta: number) {

  }

  render(ctx: CanvasRenderingContext2D) {

  }

  updateData(data: Entity) {

  }
}

class NetworkedEntity extends Entity {

}

class Bullet extends NetworkedEntity {
  targetPosition: Vector2f;

  constructor(public position: Vector2f) {
    super();

    this.targetPosition = this.position.clone();
  }

  update(delta: number) {
    const diff = this.targetPosition.clone();
    diff.x -= this.position.x;
    diff.y -= this.position.y;

    const speed = 0.7;

    this.position.x += diff.x * speed;
    this.position.y += diff.y * speed;
  }

  render(ctx: CanvasRenderingContext2D) {
    ctx.beginPath();
    ctx.fillStyle = 'red';
    ctx.arc(this.position.x, this.position.y, 5, 0, Math.PI * 2);
    ctx.fill();
  }

  updateData(data: Player) {
    this.targetPosition = new Vector2f(data.position.x, data.position.y);
  }
}

class Player extends NetworkedEntity {
  targetPosition: Vector2f;

  constructor(public position: Vector2f) {
    super();

    this.targetPosition = this.position.clone();
  }

  update(delta: number) {
    const diff = this.targetPosition.clone();
    diff.x -= this.position.x;
    diff.y -= this.position.y;

    const speed = 0.7;

    this.position.x += diff.x * speed;
    this.position.y += diff.y * speed;
  }

  render(ctx: CanvasRenderingContext2D) {
    ctx.beginPath();
    ctx.fillStyle = '#333';
    ctx.arc(this.position.x, this.position.y, 10, 0, Math.PI * 2);
    ctx.fill();
  }

  updateData(data: Player) {
    this.targetPosition = new Vector2f(data.position.x, data.position.y);
  }
}

class EntityManager {
  ids: Set<string> = new Set();
  entities: Entity[] = [];

  constructor(private game: Game) {}

  add(id: string, entity: Entity) {
    if (this.ids.has(id)) {
      return;
    }

    this.ids.add(id);
    entity.id = id;
    entity.game = this.game;
    this.entities.push(entity);
    entity.start();
  }

  remove(id: string) {
    const index = this.entities.findIndex(entity => entity.id === id);
    if (index === -1) {
      return;
    }

    this.ids.delete(id);
    this.entities[index].destroy();
    this.entities.splice(index, 1);
  }

  update(delta: number) {
    for (let i = 0; i < this.entities.length; i++) {
      this.entities[i].update(delta);
    }
  }

  render(ctx: CanvasRenderingContext2D) {
    for (let i = 0; i < this.entities.length; i++) {
      this.entities[i].render(ctx);
    }
  }

  addNetworkedEntity(id: string, data: any) {
    switch (data.kind) {
      case 'Player':
        return this.add(
          id,
          new Player(
            new Vector2f(data.position.x, data.position.y)
          )
        );
      case 'Bullet':
        return this.add(
          id,
          new Bullet(
            new Vector2f(data.position.x, data.position.y)
          )
        );
      default:
        return;
    }
  }

  updateNetworkedEntity(id: string, data: Entity) {
    const index = this.entities.findIndex(entity => entity.id === id);
    if (index === -1) {
      return;
    }

    this.entities[index].updateData(data);
  }
}

class Game {
  context: CanvasRenderingContext2D;
  provider: Provider = new Provider();
  entityManager = new EntityManager(this);
  data: { entities: { [key: string]: Entity } } = { entities: {} };
  startTime = 0;
  keyMap: { [key: string]: boolean } = {};

  count = 0;
  sumLatency = 0;

  constructor(canvas: HTMLCanvasElement) {
    this.context = canvas.getContext('2d')!;
  }

  async start() {
    await this.provider.connect();

    this.provider.on('game_state', (data: { ts: number, entities: { [key: string]: Entity } }) => {
      const addedChanges = addedDiff(this.data.entities, data.entities);
      const updatedChanges = updatedDiff(this.data.entities, data.entities);
      const deletedChanges = deletedDiff(this.data.entities, data.entities);

      this.data = data;

      for (const key in addedChanges) {
        this.entityManager.addNetworkedEntity(key, this.data.entities[key]);
      }

      for (const key in updatedChanges) {
        this.entityManager.updateNetworkedEntity(key, this.data.entities[key]);
      }

      for (const key in deletedChanges) {
        this.entityManager.remove(key);
      }

      const latency = Date.now() - data.ts;
      this.sumLatency += latency;
      this.count++;

      paper.textContent = `Latency: ${latency}ms
Average Latency: ${this.sumLatency / this.count}ms`;
    });

    this.startTime = Date.now();
    window.requestAnimationFrame(this.enterFrame);

    window.addEventListener('keydown', (e) => {
      this.keyMap[e.key] = true;
    });

    window.addEventListener('keyup', (e) => {
      this.keyMap[e.key] = false;
    });

    window.addEventListener('mousedown', (e) => {
      let x = e.pageX;
      let y = e.pageY;

      const rect = this.context.canvas.getBoundingClientRect();
      x -= rect.left;
      y -= rect.top;

      this.provider.send('fire', [x, y]);
    });
  }

  enterFrame = () => {
    const currentTime = Date.now();
    const delta = (currentTime - this.startTime) * 0.001;
    this.startTime = currentTime;

    this.context.clearRect(0, 0, 800, 600);

    for (let i = 0; i < this.entityManager.entities.length; i++) {
      this.entityManager.entities[i].update(delta);
      this.entityManager.entities[i].render(this.context);
    }

    const pos = [0, 0];

    if (this.keyMap['w']) {
      pos[1] -= 10;
    }

    if (this.keyMap['a']) {
      pos[0] -= 10;
    }

    if (this.keyMap['s']) {
      pos[1] += 10;
    }

    if (this.keyMap['d']) {
      pos[0] += 10;
    }

    if (pos[0] !== 0 || pos[1] !== 0) {
      this.provider.send('move', pos);
    }

    window.requestAnimationFrame(this.enterFrame);
  }
}

const game = new Game(canvas);

window.addEventListener('load', async () => {
  await game.start();
});