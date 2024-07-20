const url: string = "ws://localhost:3000";
const canvas: HTMLCanvasElement = document.querySelector("canvas")!;
const ctx: CanvasRenderingContext2D = canvas.getContext("2d")!;

const COLORS = ["red", "blue", "green", "yellow", "purple"] as const;

let ws: WebSocket = new WebSocket(url);
let view: DataView;
let self_idx: number;

const canvas_size: number = 255;
// const factor: number = 2;
const footer_size: number = 1;

canvas.width = canvas_size;
canvas.height = canvas_size;

ws.binaryType = "arraybuffer";

ws.onopen = () => {
    console.log("connected");
    render();
}

// not working
ws.onclose = () => {
    console.log("restarting");
    ws = new WebSocket(url);
}

ws.onmessage = (e: MessageEvent) => {

    const buf: ArrayBuffer = e.data;

    view = new DataView(buf);

    // footer 
    self_idx = view.getInt8(view.byteLength-1);

}

document.onkeydown = (e: KeyboardEvent) => {

    if(!ws.OPEN || e.repeat) {
        return;
    }

    switch(e.key) {
        case("w"):
            send_key_event(ws, 0);
            break;
        case("s"):
            send_key_event(ws, 2);
            break;
        case("a"):
            send_key_event(ws, 4);
            break;
        case("d"):
            send_key_event(ws, 6);
            break;
    }
    
};

document.onkeyup = (e: KeyboardEvent) => {

    if(!ws.OPEN) {
        return;
    }

    switch(e.key) {
        case("w"):
            send_key_event(ws, 1);
            break;
        case("s"):
            send_key_event(ws, 3);
            break;
        case("a"):
            send_key_event(ws, 5);
            break;
        case("d"):
            send_key_event(ws, 7);
            break;
    }
    
};

// we don't really need self_idx rn, could be useful in the future for some reason but i might remove it 
// coords should be sent to the server divided by the factor, server will calculate normal vector (direction)

canvas.onclick = (e: MouseEvent) => {

    const x: number = Math.floor(e.offsetX);
    const y: number = Math.floor(e.offsetY);

    const buf: ArrayBuffer = new ArrayBuffer(5);
    const view: DataView = new DataView(buf);

    view.setUint8(0, 8);
    view.setUint16(1, x);
    view.setUint16(3, y);
    
    ws.send(buf);

}

function render() {

    if(!view) {
        return requestAnimationFrame(render)
    }

    ctx.clearRect(0, 0, canvas.width, canvas.height);

    let idx: number = 0;

    while(idx < view.byteLength - footer_size) {

        switch(view.getUint8(idx)) {
            case(0): {
                user(ctx, view, idx);
                idx += 10;
                break;
            }
            case(1): {
                platform(ctx, view, idx);
                idx += 8;
                break;
            }
            case(3): {
                bullet(ctx, view, idx);
                idx += 10;
                break;
            }
            default:
                alert("Invalid Object Type")
        }

    }

    requestAnimationFrame(render);

}

function user(ctx: CanvasRenderingContext2D, view: DataView, idx: number) {
    
    let sprite_idx: number = view.getUint8(idx + 1); // change back to const
    const animation_idx: number = view.getUint8(idx + 2);
    const user_idx: number = view.getUint8(idx + 3);
    const width: number = view.getUint8(idx + 4);
    const height: number = view.getUint8(idx + 5);
    const x: number = view.getUint16(idx + 6);
    const y: number = view.getUint16(idx + 8);

    if(user_idx == self_idx) {
        sprite_idx = 2;
    }

    ctx.fillStyle = COLORS[sprite_idx];
    ctx.fillRect(x, y, width, height);

}

function platform(ctx: CanvasRenderingContext2D, view: DataView, idx: number) {
    
    const sprite_idx: number = view.getUint8(idx + 1);
    const width: number = view.getUint8(idx + 2);
    const height: number = view.getUint8(idx + 3);
    const x: number = view.getUint16(idx + 4);
    const y: number = view.getUint16(idx + 6);

    ctx.fillStyle = COLORS[sprite_idx];
    ctx.fillRect(x, y, width, height);

}

function bullet(ctx: CanvasRenderingContext2D, view: DataView, idx: number) {

    const sprite_idx: number = view.getUint8(idx + 1);

    const origin_x: number = view.getUint16(idx + 2);
    const origin_y: number = view.getUint16(idx + 4);
    const end_x: number = view.getUint16(idx + 6);
    const end_y: number = view.getUint16(idx + 8);

    console.log(end_x, end_y);

    ctx.strokeStyle = COLORS[sprite_idx];

    ctx.beginPath();

    ctx.moveTo(origin_x, origin_y);
    ctx.lineTo(end_x, end_y);

    ctx.stroke();

}

function send_key_event(ws: WebSocket, byte: number) {

    const buf: Uint8Array = new Uint8Array(1);

    buf[0] = byte;

    ws.send(buf);

}
