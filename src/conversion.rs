#![allow(dead_code)]

use bevy::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QuatStrMode {
    XYZW,
    WXYZ,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MatStrMode {
    RowMajor,
    ColMajor,
}

pub fn quat_to_strings(quat: Quat, mode: QuatStrMode) -> [String; 4] {
    let x = quat.x.to_string();
    let y = quat.y.to_string();
    let z = quat.z.to_string();
    let w = quat.w.to_string();

    match mode {
        QuatStrMode::XYZW => [x, y, z, w],
        QuatStrMode::WXYZ => [w, x, y, z],
    }
}

pub fn strings_to_quat(strings: &[String; 4], mode: QuatStrMode) -> Quat {
    let num = parse_strings_to_f32(strings);
    match mode {
        QuatStrMode::XYZW => Quat::from_xyzw(num[0], num[1], num[2], num[3]),
        QuatStrMode::WXYZ => Quat::from_xyzw(num[1], num[2], num[3], num[0]),
    }
}

pub fn vec_to_strings(vec: Vec3) -> [String; 3] {
    let x = vec.x.to_string();
    let y = vec.y.to_string();
    let z = vec.z.to_string();

    [x, y, z]
}

pub fn strings_to_vec(strings: &[String; 3]) -> Vec3 {
    let num = parse_strings_to_f32(strings);
    Vec3::new(num[0], num[1], num[2])
}

pub fn mat3_to_strings(mat: &Mat3, mode: MatStrMode) -> [String; 9] {
    let mut strings: [String; 9] = default();
    for (i, val) in mat.to_cols_array().into_iter().enumerate() {
        strings[i] = val.to_string();
    }
    if mode == MatStrMode::RowMajor {
        strings = transpose_mat_io(&strings);
    }

    strings
}

pub fn strings_to_mat3(strings: &[String; 9], mode: MatStrMode) -> Mat3 {
    match mode {
        MatStrMode::ColMajor => Mat3::from_cols_array(&parse_strings_to_f32(strings)),
        MatStrMode::RowMajor => Mat3::from_cols_array(&parse_strings_to_f32(&transpose_mat_io(strings))),
    }
}

pub fn mat4_to_strings(mat: &Mat4, mode: MatStrMode) -> [String; 16] {
    let mut strings: [String; 16] = default();
    for (i, val) in mat.to_cols_array().into_iter().enumerate() {
        strings[i] = val.to_string();
    }
    if mode == MatStrMode::RowMajor {
        strings = transpose_mat_io(&strings);
    }

    strings
}

pub fn strings_to_mat4(strings: &[String; 16], mode: MatStrMode) -> Mat4 {
    match mode {
        MatStrMode::ColMajor => Mat4::from_cols_array(&parse_strings_to_f32(strings)),
        MatStrMode::RowMajor => Mat4::from_cols_array(&parse_strings_to_f32(&transpose_mat_io(strings))),
    }
}

pub fn transpose_mat_io<const S: usize>(from: &[String; S]) -> [String; S]
where
    [String; S]: Default,
{
    let s = S.isqrt();

    let mut to: [String; S] = default();

    for i in 0..s {
        for j in 0..s {
            to[i * s + j] = from[j * s + i].clone();
        }
    }
    to
}

fn parse_strings_to_f32<const S: usize>(strings: &[String; S]) -> [f32; S] {
    let mut parsed = [0.0; S];
    for (from, to) in strings.into_iter().zip(&mut parsed) {
        *to = from.parse::<f32>().unwrap_or_default();
    }

    parsed
}
