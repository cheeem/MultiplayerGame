const ws: WebSocket = new WebSocket('ws://localhost:3000');
const canvas: HTMLCanvasElement = document.querySelector("canvas")!;
const ctx: CanvasRenderingContext2D = canvas.getContext("2d")!;

const u8_max: number = 255;
const factor: number = 2;

const footer_size: number = 2;

canvas.width = u8_max * factor;
canvas.height = u8_max * factor;

const COLORS = ["red", "blue", "green", "yellow", "purple"] as const;

//let buf: ArrayBuffer;
let buf_u8: Uint8Array;
let buf_u16: Uint16Array;

let self_idx: number;

ws.binaryType = "arraybuffer";

ws.onopen = () => {
    console.log("connected");
    render();
}

ws.onmessage = (e: MessageEvent) => {

    const buf: ArrayBuffer = e.data;

    buf_u8 = new Uint8Array(buf);
    buf_u16 = new Uint16Array(buf);

    // footer 
    self_idx = buf_u8[buf_u8.length-1];

}

document.onkeydown = (e: KeyboardEvent) => {

    if(!ws.OPEN || e.repeat) {
        return;
    }

    switch(e.key) {
        case("w"):
            send_client_event(ws, 0);
            break;
        case("s"):
            send_client_event(ws, 2);
            break;
        case("a"):
            send_client_event(ws, 4);
            break;
        case("d"):
            send_client_event(ws, 6);
            break;
    }
    
};

document.onkeyup = (e: KeyboardEvent) => {

    if(!ws.OPEN) {
        return;
    }

    switch(e.key) {
        case("w"):
            send_client_event(ws, 1);
            break;
        case("s"):
            send_client_event(ws, 3);
            break;
        case("a"):
            send_client_event(ws, 5);
            break;
        case("d"):
            send_client_event(ws, 7);
            break;
    }
    
};

// we don't really need self_idx rn, could be useful in the future for some reason but i might remove it 
// coords should be sent to the server divided by the factor, server will calculate normal vector (direction)

// canvas.onclick = (e: MouseEvent) => {

//     e.offsetX
//     e.offsetY

// }

function render() {

    if(!buf_u8) {
        return requestAnimationFrame(render)
    }

    ctx.clearRect(0, 0, canvas.width, canvas.height);

    let idx: number = 0;

    while(idx < buf_u8.length - footer_size) {

        switch(buf_u8[idx]) {
            case(0): {
                user(ctx, buf_u8, buf_u16, idx);
                idx += 10;
                break;
            }
            case(1): {
                platform(ctx, buf_u8, buf_u16, idx);
                idx += 8;
                break;
            }
        }

    }

    requestAnimationFrame(render);

}

function user(ctx: CanvasRenderingContext2D, buf_u8: Uint8Array, buf_u16: Uint16Array, idx: number) {
    
    const idx_u16: number = idx / 2;
    let sprite_idx = buf_u8[idx + 1]; // change back to const
    const animation_idx = buf_u8[idx + 2];
    const user_idx = buf_u8[idx + 3];
    const width = buf_u8[idx + 4] * factor;
    const height = buf_u8[idx + 5] * factor;
    const x = buf_u16[idx_u16 + 3] * factor;
    const y = buf_u16[idx_u16 + 4] * factor;

    if(user_idx == self_idx) {
        sprite_idx = 2;
    }

    ctx.fillStyle = COLORS[sprite_idx];
    ctx.fillRect(x, y, width, height);

}

function platform(ctx: CanvasRenderingContext2D, buf_u8: Uint8Array, buf_u16: Uint16Array, idx: number) {
    
    const idx_u16: number = idx / 2;
    const sprite_idx = buf_u8[idx + 1];
    const width = buf_u8[idx + 2] * factor;
    const height = buf_u8[idx + 3] * factor;
    const x = buf_u16[idx_u16 + 2] * factor;
    const y = buf_u16[idx_u16 + 3] * factor;

    ctx.fillStyle = COLORS[sprite_idx];
    ctx.fillRect(x, y, width, height);

}

function send_client_event(ws: WebSocket, byte: number) {

    const buf: Uint8Array = new Uint8Array(1);

    buf[0] = byte;

    ws.send(buf);

}
