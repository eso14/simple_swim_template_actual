#![no_std]


use core::intrinsics::fdiv_algebraic;

use pc_keyboard::{DecodedKey, KeyCode};
use pluggable_interrupt_os::vga_buffer::{
    is_drawable, plot, Color, ColorCode, BUFFER_HEIGHT, BUFFER_WIDTH,
};
use file_system_template::FileSystem;
//use gc_heap_template::{CopyingHeap, GenerationalHeap};
use simple_interp::{Interpreter, InterpreterOutput, ArrayString};

const NUM_WINDOWS: usize = 4;
const WINDOW_WIDTH: usize = (WIN_REGION_WIDTH - 3) / 2;
const WINDOW_HEIGHT: usize = BUFFER_HEIGHT / 2;
const DOC_HEIGHT: usize = WINDOW_HEIGHT * 4;
const TASK_MANAGER_WIDTH: usize = 10;
const WIN_REGION_WIDTH: usize = BUFFER_WIDTH - TASK_MANAGER_WIDTH;
const MAX_OPEN: usize = 16;
const BLOCK_SIZE: usize = 256;
const NUM_BLOCKS: usize = 255;
const MAX_FILE_BLOCKS: usize = 64;
const MAX_FILE_BYTES: usize = MAX_FILE_BLOCKS * BLOCK_SIZE;
const MAX_FILES_STORED: usize = 30;
const MAX_FILENAME_BYTES: usize = 10;
const MAX_TOKENS: usize = 100; 
const MAX_LITERAL_CHARS: usize = 15;
const STACK_DEPTH: usize = 20; 
const MAX_LOCAL_VARS: usize = 10; 
const HEAP_SIZE: usize = 256; 
const MAX_HEAP_BLOCKS: usize = HEAP_SIZE;

#[derive(Copy, Clone, Eq, PartialEq)]
pub struct Document {
    letters: [[char; WINDOW_WIDTH]; DOC_HEIGHT], 
    col: usize,
    row: usize,
    scroll: usize, 
}

impl Default for Document {
    fn default() -> Self {
        Self {
            letters: [[' '; WINDOW_WIDTH]; DOC_HEIGHT],
            col: 0,
            row: 0,
            scroll: 0,
        }
    }
}

/*#[derive(Copy, Clone, Eq, PartialEq)]
struct DocumentOutput {
    window: Document,
    input_buffer: ArrayString<MAX_TOKENS>,
}

impl InterpreterOutput for DocumentOutput {
    fn new(window: Document) -> Self {
        Self {
            window,
            input_buffer: ArrayString::new(),
        }
    }

    fn provide_input(&mut self, input: &str) {
        self.input_buffer.push_str(input);
    }

    fn print(&mut self, chars: &[u8]) {
        let output_str = String::from_utf8_lossy(chars);
        for (i, ch) in output_str.chars().enumerate() {
            plot(ch, self.window.x + i, self.window.y, ColorCode::new(Color::Black, Color::White));
        }
    }
}
    */

#[derive(Copy, Clone, Eq, PartialEq)]
pub struct SwimInterface {
    windows: [Document; NUM_WINDOWS],
    active_window: usize,
    prev_cursor_pos: (usize, usize), 
    fs: FileSystem,
    selected_file_index: [usize; NUM_WINDOWS],
    //programs: Vec<Interpreter<MAX_TOKENS, MAX_LITERAL_CHARS, STACK_DEPTH, MAX_LOCAL_VARS, WIN_WIDTH, CopyingHeap<HEAP_SIZE, MAX_HEAP_BLOCKS>>>,
}

impl Default for SwimInterface {
    fn default() -> Self {
        let mut fs = FileSystem::new();
        let files = [
            ("hello", r#"print("Hello, world!")"#),
            ("nums", r#"print(1)
print(257)"#),
            ("average", r#"sum := 0
count := 0
averaging := true
while averaging {
    num := input("Enter a number:")
    if (num == "quit") {
        averaging := false
    } else {
        sum := (sum + num)
        count := (count + 1)
    }
}
print((sum / count))"#),
            ("pi", r#"sum := 0
i := 0
neg := false
terms := input("Num terms:")
while (i < terms) {
    term := (1.0 / ((2.0 * i) + 1.0))
    if neg {
        term := -term
    }
    sum := (sum + term)
    neg := not neg
    i := (i + 1)
}
print((4 * sum))"#),
        ];
        for (name, content) in &files {
            let mut file = fs.open_create(name).unwrap();
            file.write(content.as_bytes()).unwrap();
            file.close().unwrap();
        }
        Self {
            windows: [Document::default(); NUM_WINDOWS],
            active_window: 0,
            prev_cursor_pos: (0, 0),
            fs,
            selected_file_index: [0; NUM_WINDOWS],
            //programs: Vec::new(),

        }
    }
}

impl SwimInterface {
    pub fn tick(&mut self) {
        self.clear_cursor(); 
        self.draw_windows();
        self.draw_cursor(); 
    }

    fn draw_windows(&self) {
        for i in 0..NUM_WINDOWS {
            let (start_x, start_y) = self.window_position(i);
            let is_active = i == self.active_window;
            let border_color = if is_active { Color::Pink } else { Color::White };

            
            for x in start_x..start_x + WINDOW_WIDTH {
                plot('.', x, start_y, ColorCode::new(border_color, Color::Black)); 
                plot('.', x, start_y + WINDOW_HEIGHT - 1, ColorCode::new(border_color, Color::Black)); 
            }
            for y in start_y..start_y + WINDOW_HEIGHT {
                plot('.', start_x, y, ColorCode::new(border_color, Color::Black)); 
                plot('.', start_x + WINDOW_WIDTH - 1, y, ColorCode::new(border_color, Color::Black)); 
            }

            
            let header_x = start_x + (WINDOW_WIDTH / 2) -1 ;
            let header_text = ['1', '2', '3', '4'][i];
            plot('F', header_x, start_y, ColorCode::new(Color::White, Color::Black));
            plot(header_text, header_x + 1, start_y, ColorCode::new(Color::White, Color::Black));

            
            let doc = &self.windows[i];
            for row in 0..WINDOW_HEIGHT {
                for col in 0..WINDOW_WIDTH {
                    let ch = doc.letters[row + doc.scroll][col];
                    plot(ch, start_x + col, start_y + row, ColorCode::new(Color::Cyan, Color::Black));
                }
            }
        let file_list = self.fs.list_files();
        let col_width = WIN_WIDTH / 3;
        let mut row = 1;
        let mut col = 0;

        for (index, filename) in file_list.iter().enumerate() {
            let x = start_x + col * col_width;
            let y = start_y + row;

            let color = if index == self.selected_file_index[i] {
                ColorCode::new(Color::Black, Color::White)
            } else {
                ColorCode::new(Color::Cyan, Color::Black)
            };
            for (j, ch) in filename.chars().enumerate() {
                plot(ch, x + j, y, color);
            }

            row += 1;
            if row >= 10 {
                row = 1;
                col += 1;
            }

        }
    }
}

    fn draw_cursor(&mut self) {
        let (start_x, start_y) = self.window_position(self.active_window);
        let doc = &self.windows[self.active_window];
        let cursor_x = start_x + doc.col;
        let cursor_y = start_y + (doc.row - doc.scroll);

        plot('_', cursor_x, cursor_y, ColorCode::new(Color::Yellow, Color::Black));

        self.prev_cursor_pos = (cursor_x, cursor_y);
    }

    fn clear_cursor(&self) {
        let (prev_x, prev_y) = self.prev_cursor_pos;
        plot(' ', prev_x, prev_y, ColorCode::new(Color::Black, Color::Black));
    }

    pub fn key(&mut self, key: DecodedKey) {
        match key {
            DecodedKey::RawKey(code) => self.handle_raw(code),
            DecodedKey::Unicode(c) => self.handle_unicode(c),
        }
    }

    fn handle_raw(&mut self, key: KeyCode) {
        match key {
            KeyCode::F1 => self.active_window = 0,
            KeyCode::F2 => self.active_window = 1,
            KeyCode::F3 => self.active_window = 2,
            KeyCode::F4 => self.active_window = 3,
            KeyCode::Backspace => self.backspace_key(),
            //KeyCode::ArrowLeft => self.move_cursor(-1, 0),
            //KeyCode::ArrowRight => self.move_cursor(1, 0),
            KeyCode::ArrowUp => self.move_cursor(0, -1),
            KeyCode::ArrowDown => self.move_cursor(0, 1),
            KeyCode::ArrowLeft => {
                if self.cursor_y[self.active_window] == 0{
                    let file_count = self.fs.list_files().len();
                if file_count > 0 {
                    self.selected_file_index[self.active_window] =
                        (self.selected_file_index[self.active_window] + file_count - 1) % file_count;
                }
                }else{
                    self.move_cursor(-1,0);
                }
                
                
            }
            KeyCode::ArrowRight => {
                if self.cursor_y[self.active_window]==0{
                    let file_count = self.fs.list_files().len();
                if file_count > 0 {
                    self.selected_file_index[self.active_window] =
                        (self.selected_file_index[self.active_window] + 1) % file_count;
                }
                }else{
                    self.move_cursor(1,0);
                    
                }
                
            },
            /*KeyCode::Char('r') => {
                let selected_file = &self.fs.list_files()[self.selected_file_index[self.active_window]];
    
                let program_text = self.fs.read_file(selected_file).unwrap();
                let mut interpreter = Interpreter::new(program_text);
    
                let doc_output = DocumentOutput {
                    window: &mut self.windows[self.active_window],
                    input_buffer: ArrayString::new(),

                };
    
                interpreter.tick(&mut doc_output);
    
                self.programs.push(interpreter);
            },
            KeyCode::F6 => {
                self.programs.clear();
            },*/
            _ => {}
        }
    }

    fn handle_unicode(&mut self, key: char) {
        if key == '\n' {
            self.enter_key();
        }
        if is_drawable(key) {
            let doc = &mut self.windows[self.active_window];
            doc.letters[doc.row][doc.col] = key;
            self.move_cursor(1, 0); 
           
        }
    }

    fn enter_key(&mut self) {
        let doc = &mut self.windows[self.active_window];
        doc.col = 0;
        doc.row += 1;

        if doc.row >= DOC_HEIGHT {
            doc.row = DOC_HEIGHT - 1; 
        }

        if doc.row >= doc.scroll + WINDOW_HEIGHT {
            doc.scroll += 1; 
        }
    }

    fn backspace_key(&mut self) {
        let doc = &mut self.windows[self.active_window];
        if doc.col > 0 {
            doc.col -= 1;
        } else if doc.row > 0 {
            doc.row -= 1;
            doc.col = WINDOW_WIDTH - 1;
        }
        doc.letters[doc.row][doc.col] = ' ';
    }

    fn move_cursor(&mut self, dx: isize, dy: isize) {
        let doc = &mut self.windows[self.active_window];

        if dx != 0 {
            if dx > 0 && doc.col + 1 < WINDOW_WIDTH {
                doc.col += 1;
            } else if dx < 0 && doc.col > 0 {
                doc.col -= 1;
            }
        }

        if dy != 0 {
            if dy > 0 {
                if doc.row + 1 < DOC_HEIGHT {
                    doc.row += 1;
                }
                if doc.row >= doc.scroll + WINDOW_HEIGHT {
                    doc.scroll += 1;
                }
            } else if dy < 0 && doc.row > 0 {
                doc.row -= 1;
                if doc.row < doc.scroll {
                    doc.scroll -= 1;
                }
            }
        }
    }

    fn window_position(&self, index: usize) -> (usize, usize) {
        let start_x = (index % 2) * WINDOW_WIDTH;
        let start_y = (index / 2) * WINDOW_HEIGHT;
        (start_x, start_y)
    }
}

