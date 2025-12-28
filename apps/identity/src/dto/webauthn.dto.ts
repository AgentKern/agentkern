/**
 * AgentKernIdentity - WebAuthn DTOs
 * 
 * Data Transfer Objects for WebAuthn endpoints.
 */

import { IsString, IsNotEmpty, IsOptional, IsObject } from 'class-validator';
import { ApiProperty, ApiPropertyOptional } from '@nestjs/swagger';

export class StartRegistrationRequestDto {
  @ApiProperty({ description: 'Principal ID' })
  @IsString()
  @IsNotEmpty()
  principalId: string;

  @ApiProperty({ description: 'User name (email or username)' })
  @IsString()
  @IsNotEmpty()
  userName: string;

  @ApiPropertyOptional({ description: 'Display name' })
  @IsOptional()
  @IsString()
  displayName?: string;
}

export class VerifyRegistrationRequestDto {
  @ApiProperty({ description: 'Principal ID' })
  @IsString()
  @IsNotEmpty()
  principalId: string;

  @ApiProperty({ description: 'Registration response from authenticator' })
  @IsObject()
  response: any;
}

export class StartAuthenticationRequestDto {
  @ApiProperty({ description: 'Principal ID' })
  @IsString()
  @IsNotEmpty()
  principalId: string;
}

export class VerifyAuthenticationRequestDto {
  @ApiProperty({ description: 'Principal ID' })
  @IsString()
  @IsNotEmpty()
  principalId: string;

  @ApiProperty({ description: 'Authentication response from authenticator' })
  @IsObject()
  response: any;
}

export class RegistrationOptionsResponseDto {
  @ApiProperty({ description: 'Registration options' })
  options: any;
}

export class AuthenticationOptionsResponseDto {
  @ApiProperty({ description: 'Authentication options' })
  options: any;
}

export class VerificationResultDto {
  @ApiProperty({ description: 'Whether verification succeeded' })
  verified: boolean;

  @ApiPropertyOptional({ description: 'Credential ID' })
  credentialId?: string;

  @ApiPropertyOptional({ description: 'Error message if failed' })
  error?: string;
}
