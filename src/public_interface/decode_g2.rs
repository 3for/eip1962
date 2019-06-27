use crate::weierstrass::twist;
use crate::weierstrass::cubic_twist;
use crate::field::{SizedPrimeField};
use crate::fp::Fp;
use crate::extension_towers::*;
use crate::extension_towers::fp2;
use crate::extension_towers::fp3;
use crate::representation::{ElementRepr, LegendreSymbol};
use crate::pairings::{frobenius_calculator_fp2, frobenius_calculator_fp3};
use crate::traits::FieldElement;
use crate::field::biguint_to_u64_vec;

use num_bigint::BigUint;
use num_traits::FromPrimitive;

use super::decode_fp::*;
use super::constants::*;

use crate::errors::ApiError;

pub(crate) fn create_fp2_extension<
    'a,
    FE: ElementRepr,
    F: SizedPrimeField<Repr = FE>,
    >
    (
        bytes: &'a [u8], 
        modulus: BigUint,
        field_byte_len: usize,
        base_field: &'a F,
    ) -> Result<(fp2::Extension2<'a, FE, F>, &'a [u8]), ApiError>
{
    if bytes.len() < EXTENSION_DEGREE_ENCODING_LENGTH {
        return Err(ApiError::InputError("Input is not long enough to get extension degree".to_owned()));
    }
    let (extension_degree, rest) = bytes.split_at(EXTENSION_DEGREE_ENCODING_LENGTH);
    if extension_degree[0] != EXTENSION_DEGREE_2 {
        return Err(ApiError::UnknownParameter("Extension degree expected to be 2".to_owned()));
    }

    let (fp_non_residue, rest): (Fp<'a, FE, F>, _) = decode_fp(&rest, field_byte_len, base_field)?;
    if fp_non_residue.is_zero() {
        return Err(ApiError::UnexpectedZero("Fp2 non-residue can not be zero".to_owned()));
    }

    {
        let modulus_minus_one_by_2 = modulus.clone() - BigUint::from_u32(1).unwrap();
        let modulus_minus_one_by_2 = modulus_minus_one_by_2 >> 1;
        let legendre = legendre_symbol(&fp_non_residue, biguint_to_u64_vec(modulus_minus_one_by_2));

        match legendre {
            LegendreSymbol::QuadraticResidue | LegendreSymbol::Zero => {
                return Err(ApiError::InputError(format!("Non-residue for Fp2 is actually a residue file {}, line {}", file!(), line!())));
            },
            _ => {}
        }
    }

    let mut extension_2 = fp2::Extension2::new(fp_non_residue);
    extension_2.calculate_frobenius_coeffs(modulus).map_err(|_| {
        ApiError::UnknownParameter("Failed to calculate Frobenius coeffs for Fp2".to_owned())
    })?;

    // // TODO: Check if need to delay until gas is estimated
    // let coeffs = frobenius_calculator_fp2(&extension_2).map_err(|_| {
    //     ApiError::UnknownParameter("Failed to calculate Frobenius coeffs for Fp2".to_owned())
    // })?;
    // extension_2.frobenius_coeffs_c1 = coeffs;
    
    Ok((extension_2, rest))
}

pub(crate) fn create_fp3_extension<
    'a,
    FE: ElementRepr,
    F: SizedPrimeField<Repr = FE>,
    >
    (
        bytes: &'a [u8], 
        modulus: BigUint,
        field_byte_len: usize,
        base_field: &'a F,
    ) -> Result<(fp3::Extension3<'a, FE, F>, &'a [u8]), ApiError>
{
    if bytes.len() < EXTENSION_DEGREE_ENCODING_LENGTH {
        return Err(ApiError::InputError("Input is not long enough to get extension degree".to_owned()));
    }
    let (extension_degree, rest) = bytes.split_at(EXTENSION_DEGREE_ENCODING_LENGTH);
    if extension_degree[0] != EXTENSION_DEGREE_3 {
        return Err(ApiError::UnknownParameter("Extension degree expected to be 3".to_owned()));
    }

    let (fp_non_residue, rest): (Fp<'a, FE, F>, _) = decode_fp(&rest, field_byte_len, base_field)?;
    if fp_non_residue.is_zero() {
        return Err(ApiError::UnexpectedZero("Fp2 non-residue can not be zero".to_owned()));
    }

    let mut extension_3 = fp3::Extension3::new(fp_non_residue);
    extension_3.calculate_frobenius_coeffs(modulus).map_err(|_| {
        ApiError::UnknownParameter("Failed to calculate Frobenius coeffs for Fp3".to_owned())
    })?;

    // let (coeffs_1, coeffs_2) = frobenius_calculator_fp3(modulus, &extension_3).map_err(|_| {
    //     ApiError::UnknownParameter("Failed to calculate Frobenius coeffs for Fp3".to_owned())
    // })?;
    // extension_3.frobenius_coeffs_c1 = coeffs_1;
    // extension_3.frobenius_coeffs_c2 = coeffs_2;
    
    Ok((extension_3, rest))
}

pub(crate) fn decode_g2_point_from_xy_in_fp2<
    'a,
    FE: ElementRepr,
    F: SizedPrimeField<Repr = FE>
    >
    (
        bytes: &'a [u8], 
        field_byte_len: usize,
        curve: &'a twist::WeierstrassCurveTwist<'a, FE, F>
    ) -> Result<(twist::TwistPoint<'a, FE, F>, &'a [u8]), ApiError>
{
    let (x, rest) = decode_fp2(&bytes, field_byte_len, curve.base_field)?;
    let (y, rest) = decode_fp2(&rest, field_byte_len, curve.base_field)?;
    
    let p: twist::TwistPoint<'a, FE, F> = twist::TwistPoint::point_from_xy(&curve, x, y);
    
    Ok((p, rest))
}

pub(crate) fn decode_g2_point_from_xy_in_fp3<
    'a,
    FE: ElementRepr,
    F: SizedPrimeField<Repr = FE>
    >
    (
        bytes: &'a [u8], 
        field_byte_len: usize,
        curve: &'a cubic_twist::WeierstrassCurveTwist<'a, FE, F>
    ) -> Result<(cubic_twist::TwistPoint<'a, FE, F>, &'a [u8]), ApiError>
{
    let (x, rest) = decode_fp3(&bytes, field_byte_len, curve.base_field)?;
    let (y, rest) = decode_fp3(&rest, field_byte_len, curve.base_field)?;
    
    let p: cubic_twist::TwistPoint<'a, FE, F> = cubic_twist::TwistPoint::point_from_xy(&curve, x, y);
    
    Ok((p, rest))
}

pub(crate) fn serialize_g2_point_in_fp2<
    'a,
    FE: ElementRepr,
    F: SizedPrimeField<Repr = FE>
    >
    (
        modulus_len: usize,
        point: &twist::TwistPoint<'a, FE, F>
    ) -> Result<Vec<u8>, ApiError>
{
    let (x, y) = point.into_xy();
    let mut result = serialize_fp2_fixed_len(modulus_len, &x)?;
    result.extend(serialize_fp2_fixed_len(modulus_len, &y)?);
    
    Ok(result)
}

pub(crate) fn parse_ab_in_fp2_from_encoding<
    'a,
    FE: ElementRepr,
    F: SizedPrimeField<Repr = FE>
    >(
        encoding: &'a [u8], 
        modulus_len: usize,
        field: &'a fp2::Extension2<'a, FE, F>
    ) -> Result<(fp2::Fp2<'a, FE, F>, fp2::Fp2<'a, FE, F>, &'a [u8]), ApiError>
{
    let (a, rest) = decode_fp2(&encoding, modulus_len, field)?;
    let (b, rest) = decode_fp2(&rest, modulus_len, field)?;

    Ok((a, b, rest))
}

pub(crate) fn parse_ab_in_fp3_from_encoding<
    'a,
    FE: ElementRepr,
    F: SizedPrimeField<Repr = FE>
    >(
        encoding: &'a [u8], 
        modulus_len: usize,
        field: &'a fp3::Extension3<'a, FE, F>
    ) -> Result<(fp3::Fp3<'a, FE, F>, fp3::Fp3<'a, FE, F>, &'a [u8]), ApiError>
{
    let (a, rest) = decode_fp3(&encoding, modulus_len, field)?;
    let (b, rest) = decode_fp3(&rest, modulus_len, field)?;

    Ok((a, b, rest))
}