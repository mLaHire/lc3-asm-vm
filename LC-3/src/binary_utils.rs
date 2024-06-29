use std::{fs::read, ops::Not};




const ISOLATE_BIT_MASK: u16 = 0b1111111111111111;
pub const MAX_MEMORY: u16 = 0xFFFF;
pub const MAX_MEMORY_SIZE: usize = 0xFFFF;
pub const WORD_WIDTH: u16 = 16;


pub enum Register{
    R0, R1, R2, R3, R4, R5, R6, R7,
}

pub struct BitRange{
    start_bit: u16,
    end_bit: u16,
}

pub type Range = (u16, u16);



impl BitRange{
    fn is_invalid(&self) -> bool{
        if self.start_bit > self.end_bit {  return true; /*panic!("Start bit > end bit. ");*/ }
        
        self.start_bit > (WORD_WIDTH-1)|| self.end_bit > (WORD_WIDTH-1)
    }

}

pub fn is_invalid(bit_range: (u16, u16)) -> bool{
    if bit_range.0 > bit_range.1 {  return true; /*panic!("Start bit > end bit. ");*/ }
    
    bit_range.0 > (WORD_WIDTH-1)|| bit_range.1 > (WORD_WIDTH-1)
}

pub fn is_valid_register(bit_range: (u16, u16)) -> bool{
    !is_invalid(bit_range) && (bit_range.1 - bit_range.0) < 3
}


enum Pow_2{
    A, B, C, D, 
    E, F, G, H, 
    I, J, K, L, 
    M, N, O, P,
}

pub fn isolate_bit(word: u16, position: u16 ) -> u16 {
    if position > 15 {
       panic!("read_bit error: 0 < n < 16, positions = {position}")
    }

    let mask = ISOLATE_BIT_MASK - (1 << position);
    (word ^ mask) & word
}


pub fn isolate_bits(word: u16, range: Range) -> u16{
    let range = BitRange{ start_bit: range.0, end_bit: range.1};
    if range.is_invalid(){
        panic!("Cannot read bit in range [{}, {}]. Maximum position: {}", 
        range.start_bit, range.end_bit, WORD_WIDTH-1);
    }

    let mut sum = 0;

    for i in range.start_bit ..=range.end_bit{
        sum += isolate_bit(word, i);
    }

    return sum;
}
//Ex, f( 0011 0000 0000 0000, 15, 12) = 0b0011
pub fn isolate_bits_then_shift(word: u16, range: (u16, u16)) -> u16{
    isolate_bits(word, range) >> range.0
}

pub fn flag_is_set(word: u16, bit: u16) -> bool{
    isolate_bits_then_shift(word, (bit, bit)) == 1
}

pub fn set_flag_true(word: &mut u16, bit: u16){
    if !flag_is_set(*word, bit){
        *word += 2 << bit;
    }
}

pub fn set_flag_false(word: &mut u16, bit: u16){
    if flag_is_set(*word, bit){
        *word -= 2 << bit;
    }
}

pub fn is_negative(word: u16) -> bool{
    flag_is_set(word, 15)
}

pub fn invert_sign(word: u16) -> u16 {
    word.not() + 1
}

pub fn add_2s_complement(word_1: u16, word_2: u16) -> u16{
    if !is_negative(word_1) && !is_negative(word_2) {
        return word_1 + word_2
    } 

    //dbg!(word_1, word_2);

    let mut carry = false;
    let mut sum = 0;

    for i in 0..WORD_WIDTH{
        //println!("{sum:160b}"); 

        let b1 = flag_is_set(word_1, i);
        let b2 = flag_is_set(word_2, i);

        if b1 != b2 {
            if !carry{
                sum += 1 << i;
            }
        }

        if (b1 && b2){
            //sum += 2 << i;
            if carry {
                sum += 1 << i;
            }

            carry = true;
            continue;
        }

        if ((!b1 && !b2)) && carry{
            sum += 1 << i;
            carry = false;
            continue;
        }
        
       
    }
    //println!("{word_1:016b} + {word_2:016b} = {sum:016b}");
   
    sum
}

pub fn add_2s_comp_to_signed(signed: i16, unsigned: u16) -> i16{
    if is_negative(unsigned) {
        let unsigned = invert_sign(unsigned);
        signed - 0i16.overflowing_add_unsigned(unsigned).0
    }else {
        signed.overflowing_add_unsigned(unsigned).0
    }
}

pub fn to_i16(word: u16) -> i16{
    if !is_negative(word) {
        word.try_into().expect("{word} is not negative")
    }else{
        let mut new_word = word.clone();
        set_flag_false(&mut new_word, 15);
        word.try_into().expect("{word} is negative")
    }
}

pub fn sign_extend(word: u16, most_significant_bit: u16) -> u16{

    if isolate_bit(word, most_significant_bit) == 0{
        return word
    }

    let mut extended: u16 = isolate_bits_then_shift(word, (0, most_significant_bit));
    
    

    for i in most_significant_bit..WORD_WIDTH{
        extended += 2 << i;
    }
    extended
}

pub mod instructions{

    use super::*;

    pub fn get_opcode_16bit(word: u16) -> u16{
        isolate_bits(word, (12,15))
    }
    
    pub fn get_opcode_4bit(word: u16) -> u16{
        isolate_bits_then_shift(word, (12,15))
    }
    
    pub fn get_register_at(word: u16, range: (u16, u16)) -> u16{
        isolate_bits_then_shift(word, range)
    }

    //From 0 to [bits] -> sext -> 16
    pub fn get_sign_ext_value(word: u16, bits: u16) -> u16 {
        sign_extend(isolate_bits_then_shift(word, (0, bits-1)), bits - 1)
    }
}

#[cfg(test)]
pub mod test{
    use super::*;

    #[test]
    pub fn flag(){
        let a = 0b0000_0000_0000_0001;
        assert_eq!(isolate_bits_then_shift(a, (0,0)), 1);
        assert!(flag_is_set(a, 0));
        assert!(!flag_is_set(a, 1));

    }

    #[test]
    pub fn add_basic(){
        let a = 0b0001;
        let b = 0b0000;


        //1 + 0 = 1
        assert_eq!(add_2s_complement(a, b), a);

        //0b0001 + 0b0001 = 0b0010
        assert_eq!(add_2s_complement(a, a), 0b0010);

        
        //negative
        

    }
    #[test]
    pub fn add_negatives(){
        let a = 0b0001;
        let b = !a + 1;

        
        

        assert!(is_negative(b));

        assert_eq!(add_2s_complement(b, a), 0);
        assert_eq!(add_2s_complement(b, a), 0);
        assert_eq!(add_2s_complement(b, 5), 4);
        
        let a = 5;
        let b = !10 + 1;
        assert!(is_negative(add_2s_complement(a, b)));

        println!("\n{:016b} + {:016b} = {:016b}", 0x300i16, -2i16, 0x300i16 -2i16);

        assert_eq!(add_2s_complement(0x300, !2+1), 0x300 - (2));
    }
}

//0110